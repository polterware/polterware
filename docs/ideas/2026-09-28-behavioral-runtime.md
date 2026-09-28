# Reprodutibilidade comportamental e runtime proposto

Status: Hipótese de pesquisa e arquitetura proposta — não implementada nem aprovada

Data: 2026-09-28

## Problema e hipótese

Determinismo da inferência significa produzir a mesma sequência de tokens. Saída estruturada significa produzir uma forma válida. **Reprodutibilidade comportamental** significa que execuções equivalentes de uma tarefa respeitam os mesmos limites e produzem efeitos operacionalmente equivalentes, ainda que o raciocínio e o texto do modelo variem. A distinção evita prometer que o LLM se tornou determinístico.

Hipótese: restringir e verificar ações observáveis fora do modelo reduz a variância das alterações, ações inesperadas e violações de contrato, com custo aceitável em sucesso, tempo e tokens. O runtime determina o conjunto de ações admissíveis; o agente propõe uma delas. O efeito só pode ser condicionado pelo Polterware quando a integração realmente medeia a execução.

## Contratos e ações

Um **Project Contract** descreve limites duradouros: caminhos protegidos, capacidades permitidas, decisões aceitas, critérios de verificação e quem pode aprovar exceções. A importação pode sugerir itens a partir de arquivos do projeto, mas fatos detectados, políticas propostas e políticas aceitas precisam permanecer distintos. Não há decisão nesta nota para criar `polter.toml` ou outro formato proprietário.

Um **Task Contract** acrescenta objetivo, escopo, riscos, ações permitidas, pós-condições e pontos de revisão para uma tarefa. Não pode enfraquecer um limite de autoridade superior sem aprovação. Deve capturar a revisão do projeto e a versão das políticas às quais se aplica; se o estado mudar, revalidar antes de executar.

**Typed actions** transformam propostas abertas em operações com parâmetros, pré-condições, efeitos e pós-condições definidos. O inventário abaixo é ilustrativo, não uma API aceita:

| Ação | Efeito relevante | Condição de exemplo |
| --- | --- | --- |
| `read_file` | Leitura | Caminho no escopo |
| `edit_file` | Escrita | Arquivo permitido e revisão ainda válida |
| `run_test` | Processo local | Comando e escopo autorizados |
| `install_dependency` | Rede e grafo de pacotes | Revisão humana ou delegação expressa |
| `git_push` | Efeito externo | Autorização explícita |

Um objeto válido por schema pode conter uma ação proibida. Validação de tipo, avaliação de política e verificação do efeito são etapas diferentes. Ações opacas, especialmente shell arbitrário, impedem classificar todos os efeitos com confiança; nesses casos a política pode exigir isolamento, revisão ou negar a ação.

## Máquina de estados e enforcement

Um fluxo possível é `UNDERSTAND → PLAN → IMPLEMENT → VERIFY → COMPLETE`, com retorno para correção quando a verificação falhar. Em `UNDERSTAND`, leitura e busca podem ser admitidas; escrita é bloqueada. Em `IMPLEMENT`, escrita segue o contrato. Em `VERIFY`, mudanças adicionais exigem nova avaliação. A sequência é uma proposta experimental: tarefas simples podem precisar de menos estados, e rigidez excessiva pode prejudicar a conclusão.

Para estado `s`, contrato de projeto `P` e contrato da tarefa `T`, o controlador calcula `Allowed(s, P, T)`. Antes de qualquer efeito mediado, a proposta recebe `ALLOW`, `REVIEW` ou `DENY`; depois, o runtime confere pós-condições e registra resultado, revisão, identidade do ator e motivo. Aprovação humana vale apenas para o escopo e a versão vistos na revisão. Falha de verificação não é sucesso, mesmo que a ação tenha sido autorizada.

O registro deve distinguir **proposta**, **decisão da política**, **aprovação**, **efeito observado** e **verificação**. Uma trilha auditável exige ordenação, integridade e retenção definidas; hash encadeado é uma opção de projeto a avaliar, não uma garantia já existente.

## Integração com a arquitetura atual

O PPM, o Context Compiler, o histórico de AgentRun e os MCP reads/escritas auditadas oferecem bases para contexto e evidência. Um broker de execução que intercepte todos os comandos seria uma expansão substancial da direção atual e não deve ser inferido dessas bases. Adaptadores podem operar em níveis distintos:

1. **Leitura e orientação:** entregar contexto e decisões, sem enforcement.
2. **Detecção posterior:** comparar alterações e apontar drift, sem impedir efeitos.
3. **Mediação parcial:** controlar apenas ações que passam por uma integração específica.
4. **Mediação completa:** controlar todas as capacidades relevantes naquele ambiente, condição necessária para alegar enforcement abrangente.

Cada resultado deve declarar seu nível. Um agente com outro caminho de escrita ou rede pode contornar um broker parcial; auditoria posterior não equivale a prevenção. O desenho precisa considerar alterações concorrentes, reentrância, falhas durante a execução, escape do sandbox e operações irreversíveis.

## Referências conceituais verificadas

- [TypeChat, Microsoft](https://github.com/microsoft/TypeChat): tradução de linguagem natural para objetos tipados e validação/reparo. Sustenta a camada estrutural, não a correção dos efeitos.
- [AgentSpec, ICSE 2026](https://conf.researchr.org/details/icse-2026/icse-2026-research-track/29/AgentSpec-Customizable-Runtime-Enforcement-for-Safe-and-Reliable-LLM-Agents): precedente de regras customizáveis verificadas em runtime para agentes. Seus resultados não devem ser extrapolados para Polterware.
- [Agent Behavioral Contracts](https://arxiv.org/abs/2602.22302): preprint de contratos comportamentais e enforcement. Tratar como proposta de pesquisa; verificar avaliação e revisão por pares antes de apoiar alegações quantitativas.
- [VIGIL: Runtime Enforcement of Behavioral Specifications in AI Agent Skills](https://arxiv.org/abs/2606.26524): preprint sobre políticas temporais e eventos de ferramentas. Há também um trabalho diferente chamado [VIGIL: Defending LLM Agents Against Tool-Stream Injection via Verify-Before-Commit](https://aclanthology.org/2026.acl-long.443/); os dois não devem ser confundidos.

## Questões abertas

- Qual subconjunto de ações cobre tarefas reais sem transformar o agente em uma interface impraticável?
- Que ações podem ser observadas mas não mediadas em cada agente externo?
- Quem define e aprova contratos, e como resolver conflito entre projeto, tarefa e política organizacional?
- Qual custo de revisão humana e qual perda de sucesso são aceitáveis?
- Como verificar pós-condições sem confundir resultado de teste com prova geral de correção?

O desenho experimental e as métricas estão em [pesquisa e validação](2026-09-28-research-and-product-validation.md).
