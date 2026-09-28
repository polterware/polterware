# Hipóteses, métricas e validação do produto

Status: Plano de pesquisa proposto — sem experimento executado ou resultado observado

Data: 2026-09-28

## Perguntas separadas

| ID | Hipótese a testar | Evidência necessária |
| --- | --- | --- |
| H1 | Coding agents variam de modo operacionalmente relevante em execuções repetidas da mesma tarefa. | Traces e estados finais em repetições controladas. |
| H2 | Ações tipadas, contratos e verificação externa reduzem variação indesejada. | Comparação entre condições com igual tarefa, base e modelo. |
| H3 | Decisões e evidências do projeto reduzem mudanças incompatíveis com arquitetura e requisitos. | Tarefas com decisões explícitas e avaliação cega de compatibilidade. |
| H4 | Cápsulas selecionadas reduzem tokens ou tempo sem perda material de sucesso. | Comparação com leitura convencional do repositório e orçamento equivalente. |
| H5 | Packs de engenharia reduzem defeitos relevantes em cenários complexos. | Falhas injetadas, invariantes e avaliação especializada por domínio. |

Essas hipóteses são independentes. Um resultado positivo em H2 não valida H3–H5. Também é possível que mais restrição aumente previsibilidade e reduza sucesso; esse trade-off é resultado de pesquisa, não falha a ocultar.

## Reprodutibilidade comportamental

Para tarefa `x`, executar `R >= 2` vezes desde o mesmo snapshot do projeto. Registrar o trace bruto `τᵢ` e uma normalização pré-especificada `φ(τᵢ)` de ações e efeitos semânticos. Por exemplo, dois comandos diferentes de leitura do mesmo arquivo podem virar `READ_FILE(path)`. A normalização não deve apagar diferenças relevantes como arquivo modificado, dependência adicionada, chamada de rede ou ordem necessária entre operações.

Definir uma distância `d` entre traces normalizados, com intervalo `[0, 1]`. A divergência média proposta é:

```text
D(x) = 2 / (R × (R - 1)) × Σᵢ<ⱼ d(φ(τᵢ), φ(τⱼ))
BR(x) = 1 - D(x)
```

`BR` alto mede semelhança segundo **esta** normalização e distância; não prova correção, segurança nem equivalência de resultados. Definir publicamente custos de inserção/remoção de ação, ordenação, parâmetros e equivalências antes de observar resultados. Validar pares de traces com julgadores humanos independentes e medir concordância entre eles; comparar seu julgamento com a métrica. Evitar uma soma ponderada única de sucesso, violação e custo sem justificativa empírica dos pesos.

Relatar em paralelo: sucesso por tarefa, pós-condições, taxa de propostas rejeitadas, violações **executadas**, ações inesperadas, arquivos/componentes tocados, mudanças de dependências, chamadas externas, reversões, intervenção humana, tokens, latência e custo. Uma tentativa bloqueada mostra pressão sobre a política; uma violação executada mostra falha de controle. Separar as duas. Medir estabilidade das decisões vigentes e diversidade nas escolhas legitimamente delegadas.

## Desenho experimental mínimo

Comparar quatro condições crescentes: (A) agente sem Polterware; (B) saída estruturada; (C) ações tipadas; (D) ações tipadas mais contratos, estados e verificação. Para H3 e H4, acrescentar ablações específicas de decisões/evidência e cápsulas, para não atribuir todos os efeitos ao runtime.

Fixar snapshot inicial, tarefa, documentação disponível, orçamento, versão de modelo, ferramentas e ambiente em cada bloco; randomizar ordem das condições e repetir tarefas. Registrar parâmetros de inferência e alterações de versão, pois igualdade de prompt não garante igualdade de tokens. Usar tarefas controladas com ações permitidas/proibidas conhecidas, tarefas reais auditadas e tarefas abertas de desenvolvimento assistido. Fazer piloto pequeno para calibrar contratos e distâncias antes de dimensionar a amostra. Os números de execuções discutidos na conversa eram exemplos de planejamento, não tamanho de amostra validado.

Preparar avaliação de falhas concretas para sistemas distribuídos e dados: duplicação e reordenação de mensagens, timeout, queda de processo, retry, concorrência, consistência e perda de atualização. Para HPC/performance, usar perfil e benchmark reproduzível antes e depois; não aceitar “código mais paralelo” como melhoria sem medida. A seleção de tarefas precisa distinguir defeitos mecânicos de escolhas arquiteturais legítimas.

Riscos de validade: tarefas contaminadas por treino, diferenças entre ferramentas, política que bloqueia a solução correta, normalização de trace enviesada, avaliadores conhecendo a condição e benchmarks que premiam apenas testes superficiais. Publicar contratos, critérios, traces sanitizados e análise de falhas quando possível. Não prometer um “nível de determinismo” único para contextos e agentes heterogêneos.

## Validação de produto antes de ampliar arquitetura

1. Entrevistar desenvolvedores e times que usam agentes para identificar incidentes concretos: decisão arquitetural silenciosa, perda de contexto, mudança fora de escopo, custo de revisão e procura por evidência.
2. Observar o fluxo atual e medir tempo para encontrar uma decisão, arquivos lidos, tokens e divergências reais. Evitar perguntas hipotéticas de intenção de compra como única prova de demanda.
3. Testar um protótipo leve de cápsula + ADR/evidência e uma revisão de decisão pendente, sem exigir gateway universal. Medir compreensão, confiança calibrada, tempo e correções realizadas.
4. Testar enforcement apenas em uma integração com caminho de ação efetivamente mediado. Medir cobertura, contornos, revisões humanas, fricção e preferência frente ao fluxo original.
5. Definir critérios de avanço para cada etapa antes dos resultados: benefício mínimo para usuários, limite de perda de sucesso, custo operacional e cobertura de ações. Critérios numéricos precisam ser escolhidos com stakeholders, não inventados nesta nota.

Posicionamento como **engineering control plane** é uma hipótese de descoberta: testar se usuários entendem e valorizam a proposta sem confundi-la com operações de infraestrutura. A categoria pública vigente continua a de [Project Context & Change Intelligence](../product-vision.md). A pesquisa pode mostrar que o valor mais forte está em contexto e evidência, sem justificar um runtime abrangente.

## Literatura inicial e limites

- [TypeChat](https://github.com/microsoft/TypeChat), [AgentSpec (ICSE 2026)](https://conf.researchr.org/details/icse-2026/icse-2026-research-track/29/AgentSpec-Customizable-Runtime-Enforcement-for-Safe-and-Reliable-LLM-Agents), [Agent Behavioral Contracts (preprint)](https://arxiv.org/abs/2602.22302) e [VIGIL: Runtime Enforcement (preprint)](https://arxiv.org/abs/2606.26524) oferecem precedentes para tipagem e controle de ações. Nenhum demonstra a hipótese específica de Polterware sem replicação no domínio de coding agents.
- [SEI QAW/ATAM](https://www.sei.cmu.edu/library/sei-architecture-analysis-techniques-and-when-to-use-them/), [ISO/IEC/IEEE 42010:2022](https://www.iso.org/standard/74393.html) e [W3C PROV-O](https://www.w3.org/TR/prov-o/) fundamentam cenários de qualidade, descrição de arquitetura e proveniência. São referências conceituais; não estudos de eficácia do produto.

A conversa mencionou possíveis locais de publicação (SBES, FORGE, ICSE, FSE, ASE e periódicos de Engenharia de Software). Escolher veículo apenas após definir contribuição e verificar edital, prazo e exigências atuais; datas citadas na conversa não são compromisso deste plano.
