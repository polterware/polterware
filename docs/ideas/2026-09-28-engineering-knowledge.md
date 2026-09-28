# Conhecimento de engenharia, evidência e cápsulas de contexto

Status: Ideia de produto e arquitetura proposta — não especificação

Data: 2026-09-28

## Objetivo

Oferecer conhecimento de engenharia consultável e aplicável a uma tarefa, sem converter “boas práticas” em ordens universais. O agente precisa receber uma afirmação curta, sua condição de aplicabilidade, evidência e lacunas; a fonte completa permanece acessível para inspeção. Esse desenho deve reforçar o PPM existente, não criar outra fonte de verdade nem um repositório proprietário de documentos.

## Quatro tipos de fonte

| Tipo | Exemplo | Limite |
| --- | --- | --- |
| Norma ou padrão | ISO, NIST, W3C, OWASP | Seu texto pode ser normativo em seu domínio, mas sua adoção pelo projeto precisa ser explícita. |
| Pesquisa científica | Papers revisados ou preprints | Resultado depende de método, contexto, qualidade e replicação. |
| Literatura e experiência de engenharia | Relatos técnicos, guias de provedores | Pode trazer prática útil sem equivaler a prova universal. |
| Evidência do projeto | ADRs, requisitos, código, testes, métricas | É a evidência mais próxima da decisão local; também pode estar incompleta ou desatualizada. |

Um registro proposto de **engineering claim** conteria: identificador, afirmação, tipo de fonte, referência e trecho verificável, data/versão, condições de aplicabilidade, limites conhecidos, grau de evidência, responsável pela curadoria, verificações possíveis e relações com decisões do projeto. O nível `advisory`, `required` ou `enforced` viria de uma política aprovada, não do prestígio da fonte. Extrações de papers e regras derivadas precisam de revisão humana antes de governar ações.

Exemplo: para um worker com mensagens duplicáveis, o sistema pode apresentar a necessidade de explicitar idempotência e estratégia de retry, com fonte e condição. Se o modelo de entrega da fila é desconhecido, registra **evidence gap**; não afirma que há exatamente-once nem exige Kafka ou outro componente. Da mesma forma, uma recomendação de performance sem perfil de CPU, memória e latência deve pedir medição antes de escolher uma otimização.

## Engineering Packs

Packs são agrupamentos **candidatos** de afirmações, verificações e fontes para problemas recorrentes. Ativação deve depender de tarefa, arquitetura observada, risco, escala e decisão humana; detectar uma tecnologia não basta para impor todo o pack.

| Pack candidato | Perguntas que pode ajudar a formular |
| --- | --- |
| Arquitetura de software | Quais atributos de qualidade e limites de componentes importam? |
| Sistemas distribuídos | Quais são as hipóteses de ordenação, consistência, falha, retry e idempotência? |
| Sistemas de dados | Qual volume, padrão de acesso, retenção, replicação e tolerância a atraso? |
| Confiabilidade | Há SLO, rastreamento e medidas de latência de cauda? |
| HPC e performance | Há perfil de gargalo, intensidade operacional e benchmark reproduzível? |
| Segurança e supply chain | Quais fronteiras de confiança, dependências e evidências de build? |
| Verificação e agentes | Quais invariantes, testes, contratos e efeitos mediáveis? |

Packs não são rótulos de qualidade. Por exemplo, um blog local não precisa receber fundamentos de consenso distribuído. Um worker de pagamentos concorrente pode precisar de evidência sobre duplicação de mensagens e efeitos idempotentes. O benefício de cada pack deve ser testado em tarefas delimitadas.

## Grafo e proveniência

Conceitualmente há três conjuntos de relações: **conhecimento** (fonte → afirmação → condição/regra), **projeto** (requisito/decisão → componente → código/teste) e **execução** (ator/tarefa → proposta → aprovação → alteração/verificação). É uma forma de consultar o PPM e suas evidências, não autorização para criar três bancos de dados separados.

Uma resposta pesquisável deveria permitir: “qual decisão rege este componente?”, “qual fonte sustenta esta recomendação?”, “qual revisão do repositório foi observada?”, “quem aprovou a exceção?” e “qual evidência falta?”. A [PROV-O do W3C](https://www.w3.org/TR/prov-o/) oferece conceitos de entidade, atividade e agente úteis para modelar origem e derivação; não exige que Polterware adote RDF.

Proveniência mínima para uma afirmação: identidade estável da fonte, versão/revisão, trecho ou localização, data da observação, ambiente/escopo, método de extração, responsável e vínculo com a conclusão. Se uma fonte contradiz código ou outra decisão, registrar conflito e drift; não substituir silenciosamente a afirmação anterior. `declared`, `observed`, `inferred` e `verified` seguem as classes da [visão do produto](../product-vision.md).

## Context Capsules e custo de tokens

Uma cápsula é uma **projeção limitada** do contexto para uma tarefa: objetivo, decisões ativas, fatos relevantes, regras aplicáveis, evidências/caminhos, conflitos e lacunas, com motivo de seleção e orçamento de tokens. A seleção pode combinar IDs e relações exatas, busca lexical e, quando necessário, recuperação semântica; o resultado de busca não é autoridade.

O Context Compiler e `context.get` já entregam pacotes limitados com trechos, motivos e referências de evidência. A ideia aqui é avaliar se decisões e conhecimento curado melhoram essa seleção, respeitando a nota de [eficiência de contexto](2026-09-28-agent-context-token-efficiency.md). Se o agente já conhece o caminho de um ADR, ler o arquivo pode ser melhor do que recebê-lo por uma ferramenta. Documentos integrais permanecem no repositório. A cápsula deve expor o que foi omitido por orçamento e permitir leitura progressiva, evitando falsa completude.

## Referências conceituais verificadas

- [ISO/IEC/IEEE 42010:2022](https://www.iso.org/standard/74393.html): vocabulário e requisitos para descrição de arquitetura; não escolhe uma arquitetura para o projeto.
- [SEI: QAW e ATAM](https://www.sei.cmu.edu/library/sei-architecture-analysis-techniques-and-when-to-use-them/): métodos para elicitar atributos de qualidade e analisar trade-offs arquiteturais.
- [W3C PROV-O](https://www.w3.org/TR/prov-o/): modelo de proveniência de entidades, atividades e agentes.
- [AgentSpec, ICSE 2026](https://conf.researchr.org/details/icse-2026/icse-2026-research-track/29/AgentSpec-Customizable-Runtime-Enforcement-for-Safe-and-Reliable-LLM-Agents): exemplo de política de agente executável; não demonstra que regras derivadas de literatura melhorarão engenharia de sistemas complexos.

Referências como Lamport, Dynamo, Dapper, *The Tail at Scale*, Roofline e QuickCheck foram sugeridas na conversa como sementes de packs. Antes de derivar qualquer regra, registrar a edição primária, a afirmação exata, o cenário e as limitações em uma revisão bibliográfica própria. Nenhum desses trabalhos valida por si só o produto Polterware.
