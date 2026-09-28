# Responsabilidade por decisões de engenharia

Status: Ideia de produto e modelo conceitual — não especificação nem política vigente

Data: 2026-09-28

## Princípio

O Polterware deve distinguir restrições objetivas, objetivos de qualidade escolhidos pelo projeto e decisões que pertencem às pessoas. Literatura técnica informa alternativas; não autoriza o agente a impor DDD, TDD, microsserviços, um design pattern, um banco de dados ou uma meta global de cobertura. Depois que o responsável decide, o sistema pode preservar o contexto, expor conflito e, quando houver regra verificável e integração adequada, aplicar a decisão.

## Classes de autoridade propostas

| Classe | Exemplo | Comportamento proposto |
| --- | --- | --- |
| Restrição aceita | Não incluir segredo no repositório; não escrever fora do escopo | Bloquear ou exigir revisão conforme política e capacidade de mediação. |
| Objetivo de qualidade | `p99 < 200 ms` para um cenário definido | Medir contra o cenário e expor resultado; a meta vem do responsável. |
| Decisão humana pendente | Monólito modular ou serviços; DDD; TDD; escopo de testes | Mostrar opções, evidência, custos e lacunas; aguardar, adiar ou receber delegação explícita. |
| Decisão humana vigente | ADR aprovado para persistência, arquitetura ou fluxo | Levar ao contexto e verificar mudanças contra seu escopo. |
| Discrição delegada | Nome de helper, organização local de código | Permitir variação dentro do orçamento de delegação. |

Nem toda restrição é universal. Mesmo “build deve passar” depende da política da tarefa, ambiente e exceções aceitas. A documentação precisa registrar quem estabeleceu a regra, em qual escopo e como ela é verificada.

## Decision record e ADR

Um registro conceitual de decisão contém pergunta, responsável, status, contexto, alternativas, critérios/atributos de qualidade, evidência do projeto e da literatura, lacunas, escolha, justificativa, consequências, escopo, política derivada e condições de revisão. Estados possíveis: `unresolved`, `proposed`, `decided`, `active`, `challenged`, `superseded`. É uma proposta de modelo, não contrato de dados implementado.

Um ADR (registro de decisão arquitetural) aprovado pode ser a fonte humana da escolha; o PPM indexa seus vínculos e proveniência. Não há razão para duplicar todo o texto do ADR em um novo formato proprietário. Uma condição de revisão pode ser, por exemplo, aumento sustentado de carga ou novo requisito multirregional. Ao atingir a condição, gerar pedido de revisão; não trocar a arquitetura automaticamente.

O método [ATAM do SEI](https://www.sei.cmu.edu/library/architecture-tradeoff-analysis-method-collection/) oferece precedente para comparar arquiteturas frente a atributos de qualidade em conflito. A [ISO/IEC/IEEE 42010:2022](https://www.iso.org/standard/74393.html) fundamenta a descrição de arquitetura em preocupações e pontos de vista de interessados. Ambos são referências conceituais, não algoritmos que escolhem a resposta correta.

## DDD, TDD, patterns e estratégia de testes

- **DDD:** oferecer quando a complexidade do domínio e a participação de especialistas tornarem a opção relevante. Expor custo de modelagem e alternativas. Não ativar por detectar entidades ou serviços.
- **TDD:** registrar como preferência de processo se o time escolher. A sequência teste–implementação–refatoração só pode ser cobrada em tarefas cobertas por essa decisão e se o trace observável permitir verificá-la.
- **Design patterns:** sugerir como candidatos com problema, contexto e consequências; não impor `Repository Pattern`, CQRS ou event sourcing como padrão global.
- **Test strategy:** o responsável define riscos, níveis de teste e critérios por domínio ou componente. Cobertura percentual isolada é uma métrica, não uma estratégia universal. A verificação observa a estratégia escolhida e explicita o que não foi executado ou não pôde ser observado.

Em todos esses casos, uma fonte científica pode apoiar uma discussão, mas não substitui requisitos, prioridades e custos locais. O projeto pode permitir que o agente recomende uma escolha; a autoridade para aceitá-la permanece explícita.

## Delegation Budget e Decision Inbox

O **Delegation Budget** é uma proposta de limite graduado para decisões: detalhe trivial pode caber ao agente; mudança de componente pode exigir registro; dependência, migração de dados, efeito externo ou decisão arquitetural pode exigir pessoa responsável. O orçamento depende de impacto, reversibilidade, incerteza, escopo e política do projeto. Não equivale a uma pontuação automática de risco universal.

Uma **Decision Inbox** poderia reunir questões abertas sem interromper a cada detalhe: mostrar responsável, prazo quando existir, alternativas, trade-offs, evidências e lacunas, além de `decidir`, `delegar` ou `adiar`. Decisões urgentes que bloqueiam a tarefa precisam voltar ao agente com estado explícito; adiar não autoriza escolha silenciosa.

Exemplo: ao criar pagamentos, o sistema pode apontar que o limite transacional e a necessidade de implantação independente são desconhecidos. O responsável escolhe monólito modular ou outra alternativa e registra por quê. Em tarefas futuras, a cápsula apresenta a decisão vigente e seus limites. Se uma premissa deixar de valer, o registro passa para revisão sem substituir a escolha anterior.

## Conexão com reprodutibilidade

O objetivo não é eliminar toda variação. Restrições aceitas e decisões vigentes devem permanecer estáveis; detalhes delegados podem variar. Assim, a pesquisa deve medir separadamente violações de invariantes, divergência de decisões e diversidade dentro do espaço delegado. Essa divisão evita considerar duas implementações locais válidas como falha só porque diferem.

Nenhuma interface ou novo estado de decisão é autorizado por esta nota. A implementação exigiria especificação própria, compatível com [Knowledge e PPM atuais](../product-vision.md).
