#!/bin/bash

echo "🔗 Aevum & Bond - Verificação Automática do Sistema"
echo "=================================================="

# Cores para output
GREEN='\033[0;32m'
RED='\033[0;31m'
BLUE='\033[0;34m'
YELLOW='\033[0;33m'
NC='\033[0m' # No Color

# Função para imprimir com cor
print_status() {
    if [ $1 -eq 0 ]; then
        echo -e "${GREEN}✅ $2${NC}"
    else
        echo -e "${RED}❌ $2${NC}"
    fi
}

# Verificar se está no diretório correto
echo -e "${BLUE}1. Verificando diretório...${NC}"
if [ -f "Cargo.toml" ] && [ -d "bond-core" ] && [ -d "shared" ]; then
    print_status 0 "Diretório correto detectado"
else
    print_status 1 "Diretório incorreto - execute este script na raiz do projeto"
    exit 1
fi

# Verificar compilação
echo -e "\n${BLUE}2. Verificando compilação...${NC}"
cargo check --workspace --quiet
print_status $? "Compilação"

# Executar testes
echo -e "\n${BLUE}3. Executando testes unitários...${NC}"
cargo test --workspace --quiet
TESTS_RESULT=$?
print_status $TESTS_RESULT "Testes unitários"

# Contar testes
if [ $TESTS_RESULT -eq 0 ]; then
    TEST_COUNT=$(cargo test --workspace --quiet 2>&1 | grep -E "test result: ok\. [0-9]+ passed" | awk '{sum+=$4} END {print sum}')
    echo -e "${GREEN}   📊 Total de testes passando: $TEST_COUNT${NC}"
fi

# Teste de performance
echo -e "\n${BLUE}4. Testando performance...${NC}"
HASHRATE=$(cargo run --quiet 2>/dev/null | grep "Taxa de hash estimada" | awk '{print $5, $6}')
if [ ! -z "$HASHRATE" ]; then
    print_status 0 "Performance de hash: $HASHRATE"
else
    print_status 1 "Falha na medição de performance"
fi

# Verificar funcionalidades principais
echo -e "\n${BLUE}5. Verificando funcionalidades...${NC}"

# Executar demonstração completa
DEMO_OUTPUT=$(cargo run --quiet 2>&1)
if echo "$DEMO_OUTPUT" | grep -q "Sprint 1 concluído com sucesso"; then
    print_status 0 "Demonstração completa"
    
    # Extrair estatísticas
    BLOCKS=$(echo "$DEMO_OUTPUT" | grep "Total de blocos" | awk '{print $4}')
    TRANSACTIONS=$(echo "$DEMO_OUTPUT" | grep "Total de transações" | awk '{print $4}')
    UTXOS=$(echo "$DEMO_OUTPUT" | grep "UTXOs ativas" | awk '{print $3}')
    SUPPLY=$(echo "$DEMO_OUTPUT" | grep "Supply total" | awk '{print $3}')
    
    echo -e "${GREEN}   🧱 Blocos criados: $BLOCKS${NC}"
    echo -e "${GREEN}   💳 Transações processadas: $TRANSACTIONS${NC}"
    echo -e "${GREEN}   🎯 UTXOs ativas: $UTXOS${NC}"
    echo -e "${GREEN}   💰 Supply total: $SUPPLY Elos${NC}"
else
    print_status 1 "Demonstração completa"
fi

# Verificar módulos implementados
echo -e "\n${BLUE}6. Verificando módulos implementados...${NC}"
MODULES=("utxo.rs" "transaction.rs" "block.rs" "mining.rs" "blockchain.rs" "hash.rs")
for module in "${MODULES[@]}"; do
    if find . -name "$module" -type f | grep -q .; then
        print_status 0 "Módulo $module"
    else
        print_status 1 "Módulo $module"
    fi
done

# Resumo final
echo -e "\n${YELLOW}📋 RESUMO DA VERIFICAÇÃO${NC}"
echo "========================="

if [ $TESTS_RESULT -eq 0 ]; then
    echo -e "${GREEN}🎉 Sistema Aevum & Bond funcionando perfeitamente!${NC}"
    echo -e "${GREEN}✅ Sprint 1: Fundação do Núcleo - COMPLETO${NC}"
    echo ""
    echo "Para executar novamente:"
    echo "  • Demonstração: cargo run"
    echo "  • Testes: cargo test --workspace"
    echo "  • Verificação: ./verificar.sh"
else
    echo -e "${RED}⚠️  Alguns problemas detectados - verifique os erros acima${NC}"
fi

echo -e "\n${BLUE}Data da verificação: $(date)${NC}"
