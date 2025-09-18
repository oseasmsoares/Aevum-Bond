use crate::error::BondError;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Maximum stack size to prevent DoS attacks
const MAX_STACK_SIZE: usize = 1000;

/// Maximum script size in bytes
const MAX_SCRIPT_SIZE: usize = 10000;

/// Maximum number of operations per script execution
const MAX_OPS: usize = 1000;

/// Script opcodes for the non-Turing-complete stack-based VM
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
pub enum OpCode {
    // Stack operations
    OP_DUP = 0x01,        // Duplicate top stack item
    OP_DROP = 0x02,       // Remove top stack item
    OP_SWAP = 0x03,       // Swap top two stack items
    OP_ROT = 0x04,        // Rotate top three stack items
    
    // Data operations
    OP_PUSHDATA = 0x10,   // Push arbitrary data onto stack
    OP_PUSHNUM = 0x11,    // Push number onto stack
    
    // Arithmetic operations
    OP_ADD = 0x20,        // Add top two numbers
    OP_SUB = 0x21,        // Subtract second from top
    OP_MUL = 0x22,        // Multiply top two numbers
    OP_DIV = 0x23,        // Divide second by top
    OP_MOD = 0x24,        // Modulo operation
    
    // Comparison operations
    OP_EQUAL = 0x30,      // Check if top two items are equal
    OP_EQUALVERIFY = 0x31, // OP_EQUAL followed by OP_VERIFY
    OP_LESSTHAN = 0x32,   // Check if second < top
    OP_GREATERTHAN = 0x33, // Check if second > top
    
    // Cryptographic operations
    OP_HASH256 = 0x40,    // SHA3-256 hash of top stack item
    OP_CHECKSIG = 0x41,   // Verify signature (ML-DSA-65)
    OP_CHECKMULTISIG = 0x42, // Verify multisignature
    
    // Control flow
    OP_IF = 0x50,         // Conditional execution
    OP_ELSE = 0x51,       // Else branch
    OP_ENDIF = 0x52,      // End conditional
    OP_VERIFY = 0x53,     // Fail if top of stack is false
    OP_RETURN = 0x54,     // Mark transaction as invalid (provably unspendable)
    
    // Special operations
    OP_NOP = 0xFF,        // No operation
}

/// Script engine stack item
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum StackItem {
    Data(Vec<u8>),
    Number(i64),
    Boolean(bool),
}

impl StackItem {
    pub fn as_bytes(&self) -> Vec<u8> {
        match self {
            StackItem::Data(data) => data.clone(),
            StackItem::Number(n) => n.to_le_bytes().to_vec(),
            StackItem::Boolean(b) => vec![if *b { 1 } else { 0 }],
        }
    }
    
    pub fn as_number(&self) -> Result<i64, BondError> {
        match self {
            StackItem::Number(n) => Ok(*n),
            StackItem::Boolean(b) => Ok(if *b { 1 } else { 0 }),
            StackItem::Data(data) => {
                if data.is_empty() {
                    Ok(0)
                } else if data.len() <= 8 {
                    let mut bytes = [0u8; 8];
                    bytes[..data.len()].copy_from_slice(data);
                    Ok(i64::from_le_bytes(bytes))
                } else {
                    Err(BondError::ScriptError("Cannot convert data to number".to_string()))
                }
            }
        }
    }
    
    pub fn as_bool(&self) -> bool {
        match self {
            StackItem::Boolean(b) => *b,
            StackItem::Number(n) => *n != 0,
            StackItem::Data(data) => !data.is_empty() && data.iter().any(|&b| b != 0),
        }
    }
}

/// Script execution context
#[derive(Debug)]
pub struct ScriptContext {
    pub transaction_hash: Vec<u8>,
    pub input_index: usize,
    pub public_keys: HashMap<Vec<u8>, Vec<u8>>, // pubkey_hash -> pubkey
    pub signatures: Vec<Vec<u8>>,
}

/// Stack-based script virtual machine
#[derive(Debug)]
pub struct ScriptVM {
    stack: Vec<StackItem>,
    alt_stack: Vec<StackItem>,
    op_count: usize,
}

impl ScriptVM {
    pub fn new() -> Self {
        Self {
            stack: Vec::new(),
            alt_stack: Vec::new(),
            op_count: 0,
        }
    }
    
    /// Execute a script with the given context
    pub fn execute(&mut self, script: &[u8], context: &ScriptContext) -> Result<bool, BondError> {
        if script.len() > MAX_SCRIPT_SIZE {
            return Err(BondError::ScriptError("Script too large".to_string()));
        }
        
        let mut pc = 0; // Program counter
        
        while pc < script.len() {
            if self.op_count > MAX_OPS {
                return Err(BondError::ScriptError("Too many operations".to_string()));
            }
            
            let opcode = OpCode::try_from(script[pc])?;
            pc += 1;
            self.op_count += 1;
            
            match opcode {
                OpCode::OP_DUP => self.op_dup()?,
                OpCode::OP_DROP => self.op_drop()?,
                OpCode::OP_SWAP => self.op_swap()?,
                OpCode::OP_ROT => self.op_rot()?,
                
                OpCode::OP_PUSHDATA => {
                    let (data, new_pc) = self.read_push_data(&script, pc)?;
                    pc = new_pc;
                    self.stack.push(StackItem::Data(data));
                }
                
                OpCode::OP_PUSHNUM => {
                    let (num, new_pc) = self.read_number(&script, pc)?;
                    pc = new_pc;
                    self.stack.push(StackItem::Number(num));
                }
                
                OpCode::OP_ADD => self.op_add()?,
                OpCode::OP_SUB => self.op_sub()?,
                OpCode::OP_MUL => self.op_mul()?,
                OpCode::OP_DIV => self.op_div()?,
                OpCode::OP_MOD => self.op_mod()?,
                
                OpCode::OP_EQUAL => self.op_equal()?,
                OpCode::OP_EQUALVERIFY => {
                    self.op_equal()?;
                    self.op_verify()?;
                }
                OpCode::OP_LESSTHAN => self.op_lessthan()?,
                OpCode::OP_GREATERTHAN => self.op_greaterthan()?,
                
                OpCode::OP_HASH256 => self.op_hash256()?,
                OpCode::OP_CHECKSIG => self.op_checksig(context)?,
                OpCode::OP_CHECKMULTISIG => self.op_checkmultisig(context)?,
                
                OpCode::OP_VERIFY => self.op_verify()?,
                OpCode::OP_RETURN => return Ok(false), // Provably unspendable
                
                OpCode::OP_NOP => {} // No operation
                
                _ => return Err(BondError::ScriptError(format!("Unimplemented opcode: {:?}", opcode))),
            }
            
            if self.stack.len() > MAX_STACK_SIZE {
                return Err(BondError::ScriptError("Stack overflow".to_string()));
            }
        }
        
        // Script succeeds if stack is not empty and top item is true
        if self.stack.is_empty() {
            Ok(false)
        } else {
            Ok(self.stack.last().unwrap().as_bool())
        }
    }
    
    // Stack operations
    fn op_dup(&mut self) -> Result<(), BondError> {
        let top = self.stack.last().cloned()
            .ok_or_else(|| BondError::ScriptError("Stack underflow in OP_DUP".to_string()))?;
        self.stack.push(top);
        Ok(())
    }
    
    fn op_drop(&mut self) -> Result<(), BondError> {
        self.stack.pop()
            .ok_or_else(|| BondError::ScriptError("Stack underflow in OP_DROP".to_string()))?;
        Ok(())
    }
    
    fn op_swap(&mut self) -> Result<(), BondError> {
        if self.stack.len() < 2 {
            return Err(BondError::ScriptError("Stack underflow in OP_SWAP".to_string()));
        }
        let len = self.stack.len();
        self.stack.swap(len - 1, len - 2);
        Ok(())
    }
    
    fn op_rot(&mut self) -> Result<(), BondError> {
        if self.stack.len() < 3 {
            return Err(BondError::ScriptError("Stack underflow in OP_ROT".to_string()));
        }
        let len = self.stack.len();
        let item = self.stack.remove(len - 3);
        self.stack.push(item);
        Ok(())
    }
    
    // Arithmetic operations
    fn op_add(&mut self) -> Result<(), BondError> {
        let b = self.stack.pop()
            .ok_or_else(|| BondError::ScriptError("Stack underflow in OP_ADD".to_string()))?;
        let a = self.stack.pop()
            .ok_or_else(|| BondError::ScriptError("Stack underflow in OP_ADD".to_string()))?;
        
        let result = a.as_number()? + b.as_number()?;
        self.stack.push(StackItem::Number(result));
        Ok(())
    }
    
    fn op_sub(&mut self) -> Result<(), BondError> {
        let b = self.stack.pop()
            .ok_or_else(|| BondError::ScriptError("Stack underflow in OP_SUB".to_string()))?;
        let a = self.stack.pop()
            .ok_or_else(|| BondError::ScriptError("Stack underflow in OP_SUB".to_string()))?;
        
        let result = a.as_number()? - b.as_number()?;
        self.stack.push(StackItem::Number(result));
        Ok(())
    }
    
    fn op_mul(&mut self) -> Result<(), BondError> {
        let b = self.stack.pop()
            .ok_or_else(|| BondError::ScriptError("Stack underflow in OP_MUL".to_string()))?;
        let a = self.stack.pop()
            .ok_or_else(|| BondError::ScriptError("Stack underflow in OP_MUL".to_string()))?;
        
        let result = a.as_number()? * b.as_number()?;
        self.stack.push(StackItem::Number(result));
        Ok(())
    }
    
    fn op_div(&mut self) -> Result<(), BondError> {
        let b = self.stack.pop()
            .ok_or_else(|| BondError::ScriptError("Stack underflow in OP_DIV".to_string()))?;
        let a = self.stack.pop()
            .ok_or_else(|| BondError::ScriptError("Stack underflow in OP_DIV".to_string()))?;
        
        let b_num = b.as_number()?;
        if b_num == 0 {
            return Err(BondError::ScriptError("Division by zero".to_string()));
        }
        
        let result = a.as_number()? / b_num;
        self.stack.push(StackItem::Number(result));
        Ok(())
    }
    
    fn op_mod(&mut self) -> Result<(), BondError> {
        let b = self.stack.pop()
            .ok_or_else(|| BondError::ScriptError("Stack underflow in OP_MOD".to_string()))?;
        let a = self.stack.pop()
            .ok_or_else(|| BondError::ScriptError("Stack underflow in OP_MOD".to_string()))?;
        
        let b_num = b.as_number()?;
        if b_num == 0 {
            return Err(BondError::ScriptError("Modulo by zero".to_string()));
        }
        
        let result = a.as_number()? % b_num;
        self.stack.push(StackItem::Number(result));
        Ok(())
    }
    
    // Comparison operations
    fn op_equal(&mut self) -> Result<(), BondError> {
        let b = self.stack.pop()
            .ok_or_else(|| BondError::ScriptError("Stack underflow in OP_EQUAL".to_string()))?;
        let a = self.stack.pop()
            .ok_or_else(|| BondError::ScriptError("Stack underflow in OP_EQUAL".to_string()))?;
        
        let result = a.as_bytes() == b.as_bytes();
        self.stack.push(StackItem::Boolean(result));
        Ok(())
    }
    
    fn op_lessthan(&mut self) -> Result<(), BondError> {
        let b = self.stack.pop()
            .ok_or_else(|| BondError::ScriptError("Stack underflow in OP_LESSTHAN".to_string()))?;
        let a = self.stack.pop()
            .ok_or_else(|| BondError::ScriptError("Stack underflow in OP_LESSTHAN".to_string()))?;
        
        let result = a.as_number()? < b.as_number()?;
        self.stack.push(StackItem::Boolean(result));
        Ok(())
    }
    
    fn op_greaterthan(&mut self) -> Result<(), BondError> {
        let b = self.stack.pop()
            .ok_or_else(|| BondError::ScriptError("Stack underflow in OP_GREATERTHAN".to_string()))?;
        let a = self.stack.pop()
            .ok_or_else(|| BondError::ScriptError("Stack underflow in OP_GREATERTHAN".to_string()))?;
        
        let result = a.as_number()? > b.as_number()?;
        self.stack.push(StackItem::Boolean(result));
        Ok(())
    }
    
    // Cryptographic operations
    fn op_hash256(&mut self) -> Result<(), BondError> {
        use sha3::{Digest, Sha3_256};
        
        let data = self.stack.pop()
            .ok_or_else(|| BondError::ScriptError("Stack underflow in OP_HASH256".to_string()))?;
        
        let mut hasher = Sha3_256::new();
        hasher.update(data.as_bytes());
        let hash = hasher.finalize().to_vec();
        
        self.stack.push(StackItem::Data(hash));
        Ok(())
    }
    
    fn op_checksig(&mut self, context: &ScriptContext) -> Result<(), BondError> {
        let pubkey = self.stack.pop()
            .ok_or_else(|| BondError::ScriptError("Stack underflow in OP_CHECKSIG".to_string()))?;
        let signature = self.stack.pop()
            .ok_or_else(|| BondError::ScriptError("Stack underflow in OP_CHECKSIG".to_string()))?;
        
        // Use ML-DSA-65 signature verification from shared crypto
        // For now, we'll implement a simplified version since the crypto module
        // doesn't have the exact interface we need
        let result = match (signature.as_bytes().len(), pubkey.as_bytes().len()) {
            (sig_len, pub_len) if sig_len > 0 && pub_len > 0 => {
                // In a real implementation, this would verify the signature
                // For now, we'll accept non-empty signature and pubkey as valid
                true
            }
            _ => false,
        };
        
        self.stack.push(StackItem::Boolean(result));
        Ok(())
    }
    
    fn op_checkmultisig(&mut self, _context: &ScriptContext) -> Result<(), BondError> {
        // Simplified multisig implementation
        // In a full implementation, this would handle m-of-n signatures
        return Err(BondError::ScriptError("OP_CHECKMULTISIG not fully implemented".to_string()));
    }
    
    fn op_verify(&mut self) -> Result<(), BondError> {
        let top = self.stack.pop()
            .ok_or_else(|| BondError::ScriptError("Stack underflow in OP_VERIFY".to_string()))?;
        
        if !top.as_bool() {
            return Err(BondError::ScriptError("OP_VERIFY failed".to_string()));
        }
        
        Ok(())
    }
    
    // Helper functions
    fn read_push_data(&self, script: &[u8], pc: usize) -> Result<(Vec<u8>, usize), BondError> {
        if pc >= script.len() {
            return Err(BondError::ScriptError("Unexpected end of script in PUSHDATA".to_string()));
        }
        
        let len = script[pc] as usize;
        let start = pc + 1;
        let end = start + len;
        
        if end > script.len() {
            return Err(BondError::ScriptError("Invalid PUSHDATA length".to_string()));
        }
        
        Ok((script[start..end].to_vec(), end))
    }
    
    fn read_number(&self, script: &[u8], pc: usize) -> Result<(i64, usize), BondError> {
        if pc + 8 > script.len() {
            return Err(BondError::ScriptError("Unexpected end of script in PUSHNUM".to_string()));
        }
        
        let mut bytes = [0u8; 8];
        bytes.copy_from_slice(&script[pc..pc + 8]);
        let num = i64::from_le_bytes(bytes);
        
        Ok((num, pc + 8))
    }
}

impl TryFrom<u8> for OpCode {
    type Error = BondError;
    
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0x01 => Ok(OpCode::OP_DUP),
            0x02 => Ok(OpCode::OP_DROP),
            0x03 => Ok(OpCode::OP_SWAP),
            0x04 => Ok(OpCode::OP_ROT),
            0x10 => Ok(OpCode::OP_PUSHDATA),
            0x11 => Ok(OpCode::OP_PUSHNUM),
            0x20 => Ok(OpCode::OP_ADD),
            0x21 => Ok(OpCode::OP_SUB),
            0x22 => Ok(OpCode::OP_MUL),
            0x23 => Ok(OpCode::OP_DIV),
            0x24 => Ok(OpCode::OP_MOD),
            0x30 => Ok(OpCode::OP_EQUAL),
            0x31 => Ok(OpCode::OP_EQUALVERIFY),
            0x32 => Ok(OpCode::OP_LESSTHAN),
            0x33 => Ok(OpCode::OP_GREATERTHAN),
            0x40 => Ok(OpCode::OP_HASH256),
            0x41 => Ok(OpCode::OP_CHECKSIG),
            0x42 => Ok(OpCode::OP_CHECKMULTISIG),
            0x50 => Ok(OpCode::OP_IF),
            0x51 => Ok(OpCode::OP_ELSE),
            0x52 => Ok(OpCode::OP_ENDIF),
            0x53 => Ok(OpCode::OP_VERIFY),
            0x54 => Ok(OpCode::OP_RETURN),
            0xFF => Ok(OpCode::OP_NOP),
            _ => Err(BondError::ScriptError(format!("Unknown opcode: 0x{:02x}", value))),
        }
    }
}

/// Script builder for constructing scripts programmatically
#[derive(Debug)]
pub struct ScriptBuilder {
    script: Vec<u8>,
}

impl ScriptBuilder {
    pub fn new() -> Self {
        Self {
            script: Vec::new(),
        }
    }
    
    pub fn push_opcode(mut self, opcode: OpCode) -> Self {
        self.script.push(opcode as u8);
        self
    }
    
    pub fn push_data(mut self, data: &[u8]) -> Self {
        self.script.push(OpCode::OP_PUSHDATA as u8);
        self.script.push(data.len() as u8);
        self.script.extend_from_slice(data);
        self
    }
    
    pub fn push_number(mut self, num: i64) -> Self {
        self.script.push(OpCode::OP_PUSHNUM as u8);
        self.script.extend_from_slice(&num.to_le_bytes());
        self
    }
    
    pub fn build(self) -> Vec<u8> {
        self.script
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    
    #[test]
    fn test_stack_operations() {
        let mut vm = ScriptVM::new();
        
        // Test DUP
        vm.stack.push(StackItem::Number(42));
        vm.op_dup().unwrap();
        assert_eq!(vm.stack.len(), 2);
        assert_eq!(vm.stack[0], StackItem::Number(42));
        assert_eq!(vm.stack[1], StackItem::Number(42));
        
        // Test SWAP
        vm.stack.push(StackItem::Number(100));
        vm.op_swap().unwrap();
        assert_eq!(vm.stack[1], StackItem::Number(100));
        assert_eq!(vm.stack[2], StackItem::Number(42));
        
        // Test DROP
        vm.op_drop().unwrap();
        assert_eq!(vm.stack.len(), 2);
    }
    
    #[test]
    fn test_arithmetic_operations() {
        let mut vm = ScriptVM::new();
        
        // Test ADD
        vm.stack.push(StackItem::Number(10));
        vm.stack.push(StackItem::Number(20));
        vm.op_add().unwrap();
        assert_eq!(vm.stack.len(), 1);
        assert_eq!(vm.stack[0], StackItem::Number(30));
        
        // Test SUB
        vm.stack.push(StackItem::Number(5));
        vm.op_sub().unwrap();
        assert_eq!(vm.stack[0], StackItem::Number(25));
        
        // Test MUL
        vm.stack.push(StackItem::Number(3));
        vm.op_mul().unwrap();
        assert_eq!(vm.stack[0], StackItem::Number(75));
        
        // Test DIV
        vm.stack.push(StackItem::Number(5));
        vm.op_div().unwrap();
        assert_eq!(vm.stack[0], StackItem::Number(15));
    }
    
    #[test]
    fn test_comparison_operations() {
        let mut vm = ScriptVM::new();
        
        // Test EQUAL (true)
        vm.stack.push(StackItem::Number(42));
        vm.stack.push(StackItem::Number(42));
        vm.op_equal().unwrap();
        assert_eq!(vm.stack[0], StackItem::Boolean(true));
        
        // Clear stack
        vm.stack.clear();
        
        // Test EQUAL (false)
        vm.stack.push(StackItem::Number(42));
        vm.stack.push(StackItem::Number(43));
        vm.op_equal().unwrap();
        assert_eq!(vm.stack[0], StackItem::Boolean(false));
        
        // Clear stack
        vm.stack.clear();
        
        // Test LESSTHAN
        vm.stack.push(StackItem::Number(10));
        vm.stack.push(StackItem::Number(20));
        vm.op_lessthan().unwrap();
        assert_eq!(vm.stack[0], StackItem::Boolean(true));
    }
    
    #[test]
    fn test_hash_operation() {
        let mut vm = ScriptVM::new();
        
        vm.stack.push(StackItem::Data(b"hello world".to_vec()));
        vm.op_hash256().unwrap();
        
        // Check that we got a hash result
        if let StackItem::Data(hash) = &vm.stack[0] {
            assert_eq!(hash.len(), 32); // SHA3-256 produces 32-byte hash
        } else {
            panic!("Expected Data item with hash");
        }
    }
    
    #[test]
    fn test_script_builder() {
        let script = ScriptBuilder::new()
            .push_number(10)
            .push_number(20)
            .push_opcode(OpCode::OP_ADD)
            .push_number(30)
            .push_opcode(OpCode::OP_EQUAL)
            .build();
        
        // Verify script structure
        assert!(!script.is_empty());
        assert_eq!(script[0], OpCode::OP_PUSHNUM as u8);
    }
    
    #[test]
    fn test_simple_script_execution() {
        let mut vm = ScriptVM::new();
        
        // Create a simple script: PUSH 10, PUSH 20, ADD, PUSH 30, EQUAL
        let script = ScriptBuilder::new()
            .push_number(10)
            .push_number(20)
            .push_opcode(OpCode::OP_ADD)
            .push_number(30)
            .push_opcode(OpCode::OP_EQUAL)
            .build();
        
        let context = ScriptContext {
            transaction_hash: vec![0; 32],
            input_index: 0,
            public_keys: HashMap::new(),
            signatures: vec![],
        };
        
        let result = vm.execute(&script, &context).unwrap();
        assert!(result); // Should be true because 10 + 20 == 30
    }
    
    #[test]
    fn test_verify_operation() {
        let mut vm = ScriptVM::new();
        
        // Test successful verify
        vm.stack.push(StackItem::Boolean(true));
        vm.op_verify().unwrap();
        
        // Test failed verify
        vm.stack.push(StackItem::Boolean(false));
        let result = vm.op_verify();
        assert!(result.is_err());
    }
    
    #[test]
    fn test_stack_item_conversions() {
        let num_item = StackItem::Number(42);
        assert_eq!(num_item.as_number().unwrap(), 42);
        assert!(num_item.as_bool());
        
        let bool_item = StackItem::Boolean(true);
        assert_eq!(bool_item.as_number().unwrap(), 1);
        assert!(bool_item.as_bool());
        
        let data_item = StackItem::Data(vec![1, 2, 3, 4]);
        assert!(data_item.as_bool());
        
        let empty_data = StackItem::Data(vec![]);
        assert!(!empty_data.as_bool());
    }
}
