# 📚 Documentação do Projeto Aevum & Bond

Este diretório contém toda a documentação técnica e conceitual do ecossistema blockchain Aevum & Bond.

## 📋 Índice da Documentação

### 📖 Documentos Principais

| Documento | Descrição | Status |
|-----------|-----------|---------|
| [`whitepaper.md`](whitepaper.md) | White Paper completo do ecossistema Aevum & Bond | ✅ Completo |
| [`especificacoes-tecnicas.md`](especificacoes-tecnicas.md) | Especificações técnicas detalhadas para implementação | ✅ Completo |

### 🛠️ Guias de Desenvolvimento

| Documento | Descrição | Status |
|-----------|-----------|---------|
| [`project-structure.md`](project-structure.md) | Estrutura e arquitetura do projeto | ✅ Completo |
| [`agile-framework.md`](agile-framework.md) | Framework Ágil com integração de IA | ✅ Completo |
| [`git-workflow.md`](git-workflow.md) | Fluxo de trabalho Git otimizado | ✅ Completo |
| [`security-performance.md`](security-performance.md) | Análise de segurança e performance | ✅ Completo |

### 🤖 Guias de IA

| Documento | Descrição | Status |
|-----------|-----------|---------|
| [`copilot-claude-prompting-guide.md`](copilot-claude-prompting-guide.md) | Guia de prompting para GitHub Copilot e Claude | ✅ Completo |
| [`copilot-prompting-guide.rs`](copilot-prompting-guide.rs) | Exemplos práticos em Rust com comentários | ✅ Completo |

## 🔍 Visão Geral do Projeto

### Aevum & Bond Ecossistema

O **Aevum & Bond** é um ecossistema financeiro de ledger duplo com arquitetura pós-quântica:

- **🏛️ Bond (BND)**: Blockchain Proof-of-Work para reserva de valor
  - Segurança máxima com ML-DSA-65
  - UTXOs Programáveis (pUTXOs)
  - Recuperação social nativa

- **⚡ Aevum (AEV)**: Blockchain DPoS para transações rápidas
  - Alto rendimento com ML-DSA-44
  - Abstração de contas nativa
  - Contratos inteligentes

### 🎯 Objetivos Principais

1. **Segurança Pós-Quântica**: Resistência a computadores quânticos desde o dia 1
2. **Experiência do Usuário**: Recuperação social elimina perda de fundos
3. **Escalabilidade**: Separação entre reserva de valor e camada transacional
4. **Sustentabilidade**: Tokenomia adaptativa com financiamento de ecossistema

## 🚀 Roadmap de Desenvolvimento

### Sprint 1 ✅ (Concluído)
- Fundação do núcleo blockchain
- Estruturas de dados básicas
- Mineração PoW funcional
- 28 testes passando

### Sprints 2-14 🔄 (Em andamento)
- Segurança pós-quântica (Sprint 2)
- Rede P2P (Sprint 3)
- Consenso descentralizado (Sprint 4)
- ... (veja [`especificacoes-tecnicas.md`](especificacoes-tecnicas.md))

## 🛠️ Stack Tecnológica

| Componente | Tecnologia | Justificativa |
|------------|------------|---------------|
| **Linguagem** | Rust 2021+ | Segurança de memória, performance |
| **Rede P2P** | rust-libp2p | Modular e agnóstico a transporte |
| **Criptografia PQC** | ML-DSA (FIPS 204) | Padrão NIST pós-quântico |
| **CLI** | clap | Interface de linha de comando idiomática |
| **Carteira Desktop** | Tauri | Segura e leve, backend em Rust |

## 📊 Métricas de Qualidade

### Sprint 1 - Status Atual
- ✅ **28 testes passando** (100% success rate)
- ✅ **Cobertura de código**: Estruturas de dados core
- ✅ **Segurança**: Análise preliminar B+ (resolução ML-DSA pendente)
- ✅ **Documentação**: ~15.000 palavras + exemplos práticos

### Análise de Segurança
- **Pontos Fortes**: Criptografia PQC, validação de transações, consenso PoW/DPoS
- **Atenção Requerida**: Finalizar implementação ML-DSA, limites de recursos
- **Recomendações**: Auditoria externa, verificação formal (longo prazo)

## 📖 Como Navegar na Documentação

### Para Desenvolvedores
1. Comece com [`project-structure.md`](project-structure.md)
2. Configure o ambiente seguindo [`agile-framework.md`](agile-framework.md)
3. Use os guias de IA para acelerar o desenvolvimento

### Para Arquitetos
1. Leia o [`whitepaper.md`](whitepaper.md) completo
2. Analise [`especificacoes-tecnicas.md`](especificacoes-tecnicas.md)
3. Revise [`security-performance.md`](security-performance.md)

### Para Contribuidores
1. Entenda o [`git-workflow.md`](git-workflow.md)
2. Use [`copilot-claude-prompting-guide.md`](copilot-claude-prompting-guide.md)
3. Siga as convenções em [`agile-framework.md`](agile-framework.md)

## 🤝 Contribuição

Esta documentação é mantida ativamente e reflete o estado atual do desenvolvimento. Para contribuições:

1. Siga o fluxo de trabalho Git documentado
2. Use as práticas de IA documentadas para acelerar o desenvolvimento
3. Mantenha a documentação atualizada com as mudanças de código
4. Execute todos os testes antes de submeter PRs

## 📞 Contato e Recursos

- **Website**: [aevum.bond](https://aevum.bond) (em desenvolvimento)
- **Repositório**: [Aevum-Bond](https://github.com/oseasmsoares/Aevum-Bond)
- **Versão**: 2.0 (Agosto 2025)

---

*Última atualização: 15 de setembro de 2025*
*Sprint 1 concluído com sucesso - avançando para implementação pós-quântica*
