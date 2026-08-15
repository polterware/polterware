# POLTERWARE — Especificação Mestre de Produto e Engenharia

> **Status:** especificação normativa
> **Versão:** 1.0
> **Data:** 2026-08-11
> **Produto principal:** Polter
> **Público:** engenheiros de software, produto, segurança, infraestrutura, QA, release engineering e agentes de IA
> **Objetivo:** servir como fonte de verdade para implementar, revisar e lançar o Polter com qualidade de produção, preservando arquitetura, segurança, regras de negócio, desempenho e consistência de UX.

---

## 0. Como usar este documento

Este documento consolida os dois documentos-base do projeto e as decisões posteriores já tomadas no projeto. Ele deve ser lido como um **contrato de implementação**: código que contradiga uma regra normativa daqui precisa de uma decisão explícita, normalmente registrada em ADR.

### 0.1 Precedência

Em caso de conflito, usar a seguinte ordem:

1. decisão explícita mais recente formalizada em especificação ou ADR;
2. este documento;
3. contratos Protocol Buffers e migrations já aceitos no repositório;
4. documentação de domínio em `docs/domains/`;
5. documentação de integrações em `docs/integrations/`;
6. implementação concreta.

Código não pode redefinir silenciosamente uma regra de produto.

### 0.2 Linguagem normativa

- **MUST / DEVE:** requisito obrigatório.
- **MUST NOT / NÃO DEVE:** comportamento proibido.
- **SHOULD / DEVERIA:** padrão esperado; exceção precisa de justificativa.
- **MAY / PODE:** opcional.

### 0.3 O que vem diretamente dos documentos-base

Os documentos-base estabelecem explicitamente:

- Polter como um control plane para pessoas e agentes de IA;
- Electron + SolidJS + Tailwind no desktop;
- backend em Go;
- `polter-core`, `polter-server`, `polter-node`, CLI e MCP;
- ConnectRPC + Protocol Buffers;
- SQLite local;
- PostgreSQL + `pgx` + `sqlc` no cloud;
- monólito modular;
- transactional outbox em PostgreSQL como mecanismo inicial de eventos confiáveis;
- NATS JetStream como evolução quando escala justificar;
- OpenTelemetry + logs estruturados;
- organização por domínios;
- providers concretos implementando interfaces definidas pelos domínios;
- árvore completa de pastas reproduzida neste documento.

Regras adicionais de segurança, consistência, observabilidade, testes, release e execução são **requisitos de production hardening** adicionados para transformar essa arquitetura em um produto lançável.

### 0.4 Polter x Polterware

Os documentos-base chamam o produto de **Polter**. Para esta especificação:

- **Polterware** é o ecossistema/namespace da organização e seus produtos;
- **Polter** é o control plane descrito aqui.

Se o repositório formalizar outra convenção, esta seção deve ser atualizada por ADR.

---

# Parte I — Produto

## 1. Definição do Polter

O Polter é um control plane de sistemas de software que reúne **arquitetura, infraestrutura, operações, conhecimento e governança** em um único modelo navegável.

A tese central do produto é:

> **A arquitetura de um sistema não deve ser apenas um diagrama. Ela deve ser um modelo vivo ligado aos recursos, providers, pessoas, agentes, evidências e operações que tornam aquele sistema real.**

O Polter permite:

- representar aplicações, serviços, APIs, bancos, filas, máquinas e dependências;
- conectar componentes arquiteturais a recursos reais;
- organizar recursos locais, VPS e cloud;
- integrar providers como AWS, Vercel, Supabase, GitHub, Grafana, Datadog e outros;
- visualizar estado, ownership, contexto e relacionamentos;
- solicitar operações locais ou remotas;
- aplicar autorização, políticas e aprovações antes da execução;
- executar por API de provider ou por Polter Node;
- transmitir logs, progresso e resultados de execuções;
- ligar documentação, decisões e runbooks aos objetos reais do sistema;
- permitir que **humanos e agents de IA co-gerenciem o mesmo project** — architecture, resources, knowledge, operations e governança — com identidade, atribuição e audit distintos;
- expor recursos e ferramentas para agentes de IA via MCP sem criar um bypass de segurança;
- registrar audit trail de alterações e ações relevantes.

## 2. O que o Polter é

O Polter é simultaneamente:

1. **workspace de arquitetura** — modelo tipado e navegável do sistema;
2. **inventário de recursos** — visão normalizada de recursos locais, remotos e cloud;
3. **hub de integrações** — camada comum sobre providers externos;
4. **superfície operacional** — ponto seguro para solicitar e acompanhar operações;
5. **base de conhecimento arquitetural** — documentação, decisões, ownership e runbooks associados ao sistema;
6. **camada de governança** — identidades, capacidades, políticas, aprovações e audit;
7. **coordenador de execução distribuída** — local, cloud e nodes remotos conectados outbound;
8. **fronteira segura para agentes** — IA opera pelas mesmas capacidades governadas usadas por humanos;
9. **workspace colaborativo humano+agent** — agents de IA são colaboradores de project de primeira classe, não apenas callers anônimos de API.

## 3. O que o Polter não é

O Polter NÃO DEVE tentar substituir integralmente:

- AWS, GCP, Azure, Vercel, Supabase ou Cloudflare;
- Grafana, Datadog, Prometheus e suítes completas de observabilidade;
- Terraform e outras ferramentas de Infrastructure as Code;
- Kubernetes;
- GitHub Actions, GitLab CI e plataformas completas de CI/CD;
- ferramentas especializadas de administração de banco;
- soluções completas de privileged access management;
- remote desktop genérico;
- Notion, Miro ou ferramentas generalistas de documentação/whiteboard.

O Polter oferece operações comuns, contexto normalizado e navegação integrada. Configuração avançada deve continuar disponível no produto original através de deep links quando fizer mais sentido.

### 3.1 Regra de escopo

Uma funcionalidade só pertence ao Polter se resolver pelo menos um destes problemas:

- entendimento da arquitetura;
- entendimento de recursos;
- operação segura;
- sincronização;
- governança;
- colaboração;
- conhecimento;
- continuidade operacional.

Não implementar uma capability apenas porque um provider possui uma API para ela.

## 4. Princípios do produto

### 4.1 O canvas é um modelo operacional

Um card no canvas não é decoração. Sempre que aplicável, ele deve poder resolver para recursos, ownership, knowledge, status, integrações, logs, operações e audit history.

### 4.2 Providers continuam autoritativos sobre estado que pertence a eles

Se AWS, Vercel ou Supabase são donos daquele estado, o Polter não deve fingir ser a nova autoridade. O Polter normaliza e referencia esse estado.

### 4.3 O Polter é autoridade sobre seu próprio contexto

O Polter é autoritativo para:

- modelo de arquitetura;
- organização de projetos e environments;
- políticas;
- aprovações;
- operation requests;
- workflows;
- permissões/delegações de agents;
- knowledge gerenciado no Polter;
- audit records;
- vínculos entre objetos Polter e recursos externos.

### 4.4 Least privilege por padrão

Usuários, agents, integrações e nodes devem receber apenas as capacidades necessárias.

### 4.5 Nenhuma permissão privilegiada fica somente no frontend

Renderer, CLI e MCP podem ocultar/desabilitar ações por UX, mas enforcement real ocorre no backend.

### 4.6 Humanos e agentes seguem a mesma governança

Agent de IA nunca recebe caminho oculto que ignore authorization, policies, approvals ou audit.

### 4.7 Local e cloud são complementares

Desktop e `polter-core` precisam ter valor local. O cloud habilita colaboração, organizações, coordenação, shared state, approvals, billing e nodes distribuídos.

### 4.8 Estado explícito vence automação mágica

Offline, stale, partial, denied, waiting approval, failed e unknown são estados legítimos e devem aparecer como tal.

### 4.9 Capabilities, não `if provider === ...`

Frontend e backend devem operar sobre capabilities normalizadas sempre que possível.

### 4.10 Falha é parte do domínio

Timeout, rate limit, disconnect e resultado indeterminado não são exceções de UI; são estados esperados do sistema distribuído.

### 4.11 Humanos e agents co-gerenciam o project

O Polter NÃO trata agents apenas como callers de automação. Agents de IA com capabilities no project são **colaboradores de primeira classe**: compartilham o mesmo modelo de architecture, resources, knowledge, operations e governança que os humanos.

O produto DEVE:

- permitir registrar, delegar e revogar agent identities no escopo organization/project;
- expor superfícies humanas (Access, Activity, Audit, Inbox) para administrar e inspecionar a colaboração humano+agent;
- expor MCP (e CLI quando aplicável) para que o agent execute as mesmas capabilities de domínio autorizadas;
- atribuir cada mutação ao actor correto (`person` | `agent` | `system`), com principal delegador quando houver delegation;
- aplicar a mesma policy/approval path a ações equivalentes de humanos e agents (§4.6).

O produto NÃO DEVE:

- criar caminho privilegiado para agent que ignore authorization, policy, approval ou audit;
- permitir que agent conceda capabilities a si mesmo;
- apagar a distinção de actor no audit (`agent change` ≠ `human change` ≠ `provider-observed change`).

Co-gestão NÃO significa autonomia irrestrita em produção: human approval continua obrigatório quando policy exigir. MCP é a interface programática primária; Desktop Access/Activity/Audit são as superfícies humanas para gerir e observar essa colaboração.

---

## 5. Atores

### 5.1 Humanos

- **Organization Owner:** autoridade máxima da organização.
- **Administrator:** gerencia members, projects, integrations e acesso dentro do escopo permitido.
- **Operator:** solicita/executa operações permitidas.
- **Member:** colaborador normal de projeto.
- **Viewer:** acesso somente leitura.
- **Reviewer:** capacidade de decidir approvals; pode coexistir com outro papel.

A implementação deve ser baseada em **capabilities**. Roles são agrupadores convenientes, não a regra final de autorização.

Capabilities equivalentes às de Member/Operator/Viewer/Reviewer PODEM ser concedidas a agents via o mesmo modelo de capability; o agent não reutiliza o papel humano como persona de UI, mas opera no mesmo project workspace.

### 5.2 Não humanos

- **Agent Identity:** identidade de IA/automação e **colaborador de project/org de primeira classe**, com capabilities delegadas; ações atribuídas ao agent e, quando aplicável, ao principal delegador; aparece em Access (Agents), Activity e Audit como ator distinto de Person.
- **Service Identity:** identidade machine-to-machine.
- **Polter Node:** agente de execução instalado em uma máquina.
- **Provider Connection:** conexão autenticada com plataforma externa.
- **CLI Session:** ator autenticado via terminal.
- **MCP Session:** sessão de agent com ferramentas/recursos limitados por autorização.

## 6. Hierarquia canônica

Requisito de produção:

```text
Organization
└── Project
    ├── Environment(s)
    ├── System(s)
    │   ├── Architecture Components
    │   ├── Connections / Dependencies
    │   ├── Resource Links
    │   ├── Knowledge Links
    │   └── Ownership
    ├── Resources
    ├── Integrations
    ├── Nodes
    ├── Operations / Workflows
    ├── Approvals / Policies
    └── Audit / Knowledge
```

`Workspace` permanece um conceito suportado pelo domínio definido nos documentos-base, mas não deve ser promovido a mais uma hierarquia visual sem necessidade. Na UI atual, Organization e Project são os principais contêineres visíveis.

## 7. Environments

Cada environment deve possuir:

- ID estável;
- `project_id`;
- nome;
- label/tipo opcional;
- token/cor de identificação visual;
- status de lifecycle;
- timestamps;
- mapeamentos opcionais de providers.

`local`, `development`, `preview`, `staging` e `production` são valores comuns, mas environments customizados devem ser permitidos.

Policies podem aplicar regras mais rígidas a environments protegidos.

---

## 8. Information architecture do produto

Sidebar atual:

```text
Overview
Architecture
Resources
Operations
Inbox
Terminal
Knowledge
Access
Integrations
Organizations
Projects
────────────
Settings
```

### 8.1 Overview

Deve resumir o contexto atual sem virar uma tela de configuração redundante:

- project/environment atual;
- visão rápida do sistema;
- mudanças recentes;
- operações recentes;
- approvals pendentes;
- saúde de nodes/integrations;
- atalhos de alta frequência.

As antigas tabs `Summary`/`Activity` não são requisito, pois não foram confirmadas como estrutura final.

### 8.2 Architecture

Tabs obrigatórias:

- **Canvas**;
- **Components**;
- **Views**.

Canvas e Components operam sobre o mesmo modelo canônico. Um node visual não é uma cópia independente do componente.

### 8.3 Resources

Tabs obrigatórias:

- **Inventory**;
- **Nodes**;
- **Processes**.

### 8.4 Operations

Tabs obrigatórias:

- **Runs**;
- **Workflows**.

### 8.5 Inbox

Tabs obrigatórias:

- **Pending**;
- **Approved**;
- **Rejected**.

Estados devem ser diferenciados por texto/ícone e também por cor, sem depender somente de cor.

### 8.6 Terminal

O Terminal é interface sobre o domínio `operations` e runtime de terminal. Não pode bypassar policies/audit.

Deve suportar evolutivamente:

- sessão local;
- sessão por node remoto;
- stdout/stderr;
- resize;
- sinais/cancelamento;
- close;
- correlation/audit;
- credenciais opacas ao renderer.

### 8.7 Knowledge

Knowledge armazena conhecimento ligado ao projeto:

- documentos;
- decisões arquiteturais;
- runbooks;
- ownership;
- links/relações com systems, components, resources e operations;
- indexação e busca.

A UI deve priorizar uma busca clara e filtros contextuais. Evitar empilhar tabs + search + múltiplas barras permanentes de filtros sem necessidade.

### 8.8 Access

É a composição visual dos domínios:

- identity;
- organizations/memberships;
- agents;
- policies;
- approvals/reviewer eligibility;
- audit relacionado a acesso.

Access é a superfície humana para **People e Agents** no project: registrar/convidar, delegar capabilities, revogar e inspecionar memberships/delegations. Agents e pessoas co-gerenciam o project sob o mesmo motor de autorização; a feature Access não inventa um segundo path de privilege.

Tabs típicas no project: **People · Agents · Policies · Audit**. Activity (filtros People/Agents) e Audit distinguem mutações por kind de actor.

Não criar uma segunda engine de autorização dentro da feature Access.

### 8.9 Integrations

Cada integração conectada deve apresentar:

- provider;
- health;
- scopes/capabilities;
- última sincronização bem-sucedida;
- stale/error state;
- projects/environments afetados;
- reconnect/revoke;
- deep link para gerenciamento externo quando aplicável.

### 8.10 Organizations

Gerencia organização, ownership, teams, memberships e invitations.

### 8.11 Projects

Gerencia criação/seleção/arquivamento de projects e seu contexto.

### 8.12 Settings

Tabs confirmadas:

- **General**;
- **Environments**.

---

# Parte II — Arquitetura do Sistema

## 9. Topologia de alto nível

```text
Pessoa / Agent
      │
      ├───────────────┬───────────────┐
      ▼               ▼               ▼
Electron Desktop     CLI              MCP
      │               │               │
      └───────────────┴───────────────┘
                      │
                typed contracts
                      │
           ┌──────────┴──────────┐
           ▼                     ▼
      polter-core           polter-server
      local runtime          cloud runtime
           │                     │
           ├──── providers ──────┤
           │                     │
           └──────────┬──────────┘
                      ▼
                  polter-node
               outbound connection
                      │
                execução remota
                      │
              logs/status/result
                      │
                 audit + sync
```

## 10. Runtimes

### 10.1 `polter-core`

Backend Go local supervisionado pelo Electron.

Responsabilidades:

- estado local;
- SQLite;
- local discovery;
- operações locais;
- providers locais quando apropriado;
- cache e materialized state;
- sync com cloud;
- offline queue;
- runtime de processo/terminal/filesystem;
- audit local e correlação com cloud.

`polter-core` não é um conjunto de helpers do Electron. É um runtime de backend independente.

### 10.2 `polter-server`

Backend cloud do Polter.

Responsabilidades:

- organizations/memberships;
- projects compartilhados;
- sessions/auth coordination;
- PostgreSQL;
- policies;
- approvals;
- audit;
- integrations cloud;
- node registry/dispatch;
- workflows compartilhados;
- notifications;
- synchronization;
- billing/entitlements.

Deve iniciar como monólito modular.

### 10.3 `polter-node`

Agente instalado em local/VPS/server para execução remota.

Responsabilidades:

- registro;
- identidade;
- conexão outbound;
- advertise de capabilities;
- heartbeat;
- dispatch;
- runtime local restrito;
- stream de output/progress;
- cancellation/signals;
- resultado;
- upgrade controlado.

Node não deve abrir porta de controle inbound ampla por padrão.

### 10.4 CLI

Cliente de primeira classe que utiliza os mesmos contracts e a mesma autorização do restante do produto.

Deve oferecer:

- consultas de architecture/resources;
- login/auth;
- operation requests;
- acompanhamento de runs;
- configuração suportada;
- execução scriptável com output machine-readable estável.

### 10.5 MCP

Interface programática primária para agents de IA **co-gerenciarem e operarem o project** com o mesmo contrato de domínio da UI.

Deve:

- mapear tools/resources para capabilities dos domínios (architecture, resources, knowledge, operations, access metadata e demais superfícies autorizadas — não só automação destrutiva);
- autenticar e atribuir actor (agent identity + session);
- autorizar cada tool call;
- aplicar policies;
- exigir approval quando uma operação humana equivalente exigiria;
- redigir secrets;
- auditar atividade com distinção de actor e, quando houver, principal delegador;
- nunca expor generic unrestricted shell por padrão;
- nunca criar bypass de governança em relação a humanos na mesma capability.

## 11. Stack backend

```text
Go
├── API: ConnectRPC + Protocol Buffers
├── Local DB: SQLite
├── Cloud DB: PostgreSQL + pgx + sqlc
├── Arquitetura: monólito modular
├── Eventos iniciais: PostgreSQL transactional outbox
├── Escala de eventos: NATS JetStream
└── Observabilidade: OpenTelemetry + logs estruturados
```

### 11.1 Monólito modular

Domínios devem ser isolados por interfaces mesmo dentro do mesmo processo. Um domínio não acessa o banco de outro por implementação concreta nem importa provider concreto.

### 11.2 Contracts

Protocol Buffers são a fonte canônica de tipos cross-runtime. Go e TypeScript gerados não podem ser mantidos manualmente em paralelo.

### 11.3 Persistência

- local: SQLite;
- cloud: PostgreSQL;
- cloud SQL: `pgx` + `sqlc`;
- domains dependem de repository ports, não de driver concreto.

### 11.4 Eventos

Quando mutation de banco e publicação de evento precisam ser atomicamente coordenadas, usar transactional outbox.

Delivery é tratada como **at-least-once**; consumidores precisam ser idempotentes.

NATS JetStream é uma evolução, não dependência direta dos domínios.

## 12. Stack frontend

```text
Electron
├── Main
├── Preload
└── Renderer
    └── SolidJS
        ├── Tailwind CSS v4
        ├── HTML semântico nativo
        ├── Ark UI
        ├── Corvu
        ├── TanStack Virtual
        └── Web Workers
```

### 12.1 Ark UI substitui Kobalte

O primeiro documento-base citava Kobalte. Decisão posterior do projeto migrou as primitives para **Ark UI**, mantendo o styling do Polter.

Norma atual:

- novos componentes acessíveis complexos usam Ark UI quando houver primitive adequada;
- Kobalte não deve entrar em código novo;
- não duplicar primitive bem implementada com solução custom sem justificativa;
- Ark UI fornece comportamento, não identidade visual.

### 12.2 Electron Main

Responsável por:

- lifecycle;
- windows;
- menus;
- native integration;
- application permissions;
- `backend-supervisor`;
- updates;
- IPC handlers seguros.

Não concentrar business logic.

### 12.3 Preload

Única ponte de privilégio renderer → Electron.

Deve:

- expor API mínima e tipada;
- allowlist de channels;
- validar payloads;
- nunca expor `ipcRenderer` genérico;
- nunca expor `fs`, `process`, `child_process` ou equivalente ao renderer.

### 12.4 Renderer

Responsável por:

- shell/routes;
- UI;
- canvas;
- logs/terminal visualization;
- cache e view state;
- acessibilidade;
- keyboard interaction.

Não executa operações privilegiadas diretamente.

### 12.5 Workers

Usar para:

- graph layout;
- parsing pesado;
- busca/filtro em logs grandes;
- diff;
- indexação;
- transforms custosos.

Workers não são banco paralelo de estado canônico.

## 13. Baseline de segurança Electron

Requisito de produção:

- `contextIsolation: true`;
- `nodeIntegration: false` no renderer;
- sandbox quando compatível;
- CSP restritiva;
- nenhuma navegação remota arbitrária em janela privilegiada;
- deny de origins inesperadas em navigation/new-window;
- preload mínimo;
- validação de IPC no lado privilegiado;
- renderer tratado como input não confiável;
- secrets não retornam ao renderer quando reference/token opaco basta;
- nenhuma concatenação de command string baseada em input de UI;
- external URLs validadas;
- updates assinados/verificados;
- telemetry/crash reporting com redaction.

## 14. Fluxo do cliente

```text
Route / Feature
      ↓
Frontend Application Service
      ↓
Typed Contract Client
      ├── local RPC → polter-core
      └── cloud RPC → polter-server
                       ↓
                 domain logic
                       ↓
              repository/provider port
                       ↓
             DB / provider / node
```

A UI não precisa saber se uma ação termina em SQLite, PostgreSQL, AWS, Docker ou Polter Node.

---

# Parte III — Arquitetura de Domínios

## 15. Grupos de domínios

```text
Core do produto
├── organizations
├── projects
├── architecture
├── resources
└── knowledge

Operação
├── provisioning
├── operations
├── workflows
├── terminal (composição sobre operations/runtime)
└── logs

Segurança e governança
├── identity
├── members (composição sobre identity/organizations)
├── agents
├── approvals
├── policies
└── audit

Infraestrutura distribuída
├── nodes
├── integrations
├── synchronization
├── observability
└── notifications

Negócio cloud
├── billing
├── plans
├── subscriptions
└── entitlements
```

No backend, `members` não precisa de bounded context independente: é composição de `identity` + `organizations`. Da mesma forma, `terminal` é feature/runtime sobre `operations` e `platform/runtime/terminal`.

## 16. Direção obrigatória das dependências

```text
apps
  ↓
features
  ↓
frontend application
  ↓
contracts

backend runtimes
  ↓
compositions
  ↓
domains
  ↓
platform abstractions

providers
  ↓
interfaces definidas pelos domains

domains
  ✕ não dependem de apps
  ✕ não dependem de providers concretos
  ✕ não dependem de Electron
  ✕ não dependem da interface web
```

Regras adicionais:

- `backend/domains/**` NÃO DEVE importar `backend/providers/**`;
- provider concreto implementa port definido pelo domínio consumidor;
- comunicação domain-to-domain deve ser explícita por application service/event;
- runtime/composition faz dependency injection;
- circular dependency entre domínios é bug arquitetural.

---

## 17. `identity`

### Responsabilidades

- users;
- sessions;
- credentials;
- service identities;
- authentication.

### Invariantes

- toda request autenticada resolve para um actor único;
- human, service e agent identities devem ser distinguíveis;
- revogação de session precisa ter efeito previsível;
- credential secret nunca aparece em DTO geral;
- authentication prova identidade; authorization é etapa separada.

### Eventos sugeridos

```text
identity.user_created
identity.session_started
identity.session_revoked
identity.credential_rotated
identity.service_identity_created
```

## 18. `organizations`

### Responsabilidades

- organizations;
- memberships;
- teams;
- invitations.

### Invariantes

- membership pertence a exatamente uma organization e uma identity;
- organization ativa não pode ficar sem owner;
- invitation expira e não pode ser reutilizada indefinidamente;
- team não concede capability fora da organization;
- delete de organization é lifecycle controlado, não cascade cego.

## 19. `projects`

### Responsabilidades

- projects;
- workspaces;
- environments;
- ownership.

### Invariantes

- project pertence a uma organization;
- environment pertence a um project;
- production/protected environment pode receber policy default mais rígida;
- ownership precisa apontar para actor/team válido;
- archive preserva audit/history.

## 20. `architecture`

### Responsabilidades

- systems;
- components;
- connections;
- dependencies;
- boundaries;
- views;
- layouts;
- snapshots.

### Modelo

- **System:** raiz lógica de arquitetura dentro do project.
- **Component:** elemento arquitetural tipado.
- **Connection:** relação/fluxo explícito entre componentes.
- **Dependency:** dependência semântica.
- **Boundary:** agrupamento/fronteira arquitetural.
- **View:** projeção/filtro salvo do mesmo modelo.
- **Layout:** estado visual de posicionamento.
- **Snapshot:** captura imutável para history/diff/review.

### Invariantes

- remover um node visual não pode deletar recurso externo implicitamente;
- layout não altera semântica do componente;
- connection referencia endpoints válidos;
- component type e resource type não são o mesmo conceito;
- várias views apontam para os mesmos componentes canônicos;
- snapshot é imutável;
- vínculo architecture ↔ resource registra provenance;
- Canvas e Components editam o mesmo aggregate lógico.

### Regra de canvas

É proibido manter o canvas como um JSON independente que não possa ser reconciliado com Components/Resources.

## 21. `resources`

### Responsabilidades

- resources;
- resource types;
- relationships;
- inventory;
- discovery;
- status;
- metadata.

### Forma normalizada sugerida

```text
Resource
├── id
├── project_id
├── environment_id?
├── provider_connection_id?
├── provider_resource_id?
├── resource_type
├── display_name
├── canonical_name?
├── status
├── metadata
├── labels/tags
├── external_url?
├── discovered_at?
├── last_seen_at?
├── source/provenance
└── timestamps
```

### Invariantes

- provider ID estável deve preservar identidade entre syncs;
- discovery é idempotente;
- resource sumido não vira `deleted` imediatamente;
- `stale`, `missing` e `deleted` precisam ser estados distinguíveis;
- raw provider metadata não pode contaminar toda a UI genérica;
- destructive action sobre resource é `operation`, não `UPDATE resources SET ...`;
- annotations do usuário não devem ser sobrescritas por refresh do provider.

## 22. `integrations`

### Responsabilidades

- connections;
- providers;
- credentials;
- capabilities;
- synchronization;
- webhooks.

### Capability model

Todo adapter deve declarar capabilities explícitas, por exemplo:

```text
discover_resources
read_resource
read_status
read_logs
stream_logs
create_resource
update_resource
delete_resource
execute_action
open_external
receive_webhook
sync_incremental
```

A UI consulta capabilities; não assume paridade funcional entre providers.

### Credenciais

- persistir secret encrypted/reference somente quando necessário;
- não registrar secret em logs, audit ou analytics;
- API comum retorna opaque reference, nunca secret plaintext quando não necessário;
- reconnect/revoke/rotation precisam ser suportados.

## 23. `provisioning`

### Responsabilidades

- definitions;
- plans;
- executions;
- state;
- drift;
- rollback.

### Regra central

Provisioning deve ser **plan-driven** quando o provider permitir inspeção útil antes da mutation.

### State machine conceitual

```text
DRAFT
  ↓
PLANNED
  ↓
POLICY_EVALUATION
  ├── DENIED
  ├── APPROVAL_REQUIRED → WAITING_APPROVAL
  └── ALLOWED
                    ↓
                 RUNNING
        ┌───────────┼───────────┐
        ▼           ▼           ▼
    SUCCEEDED     FAILED     CANCELLED

                      └── ROLLBACK_AVAILABLE / ROLLED_BACK
```

Rollback nunca deve ser prometido quando provider/action não possui reversão segura.

## 24. `operations`

### Responsabilidades

- commands;
- executions;
- sessions;
- streams;
- results;
- cancellation;
- scheduling.

### Lifecycle obrigatório

```text
intent
  ↓
validate
  ↓
authorize
  ↓
evaluate policy
  ↓
request approval se necessário
  ↓
dispatch
  ↓
execute
  ↓
stream progress/output
  ↓
complete / fail / cancel
  ↓
result + audit
```

### Invariantes

- request é descrição imutável de intent depois de dispatch/approval binding;
- retry não pode duplicar mutation destrutiva sem controle;
- mutations usam idempotency key quando aplicável;
- cancellation é best-effort e precisa dizer se underlying action realmente parou;
- stdout/stderr/progress usam backpressure;
- toda execução possui correlation ID;
- UI, CLI e MCP usam a mesma API de operations;
- unknown external outcome não deve ser falsamente convertido em `FAILED`.

### Status sugeridos

```text
REQUESTED
VALIDATING
POLICY_EVALUATION
WAITING_APPROVAL
QUEUED
DISPATCHING
RUNNING
CANCELLING
SUCCEEDED
FAILED
CANCELLED
DENIED
EXPIRED
LOST
```

`LOST` significa que o Polter não consegue afirmar com segurança o resultado final da execução.

## 25. `workflows`

### Responsabilidades

- definitions;
- steps;
- runs;
- triggers;
- conditions;
- retries;
- artifacts.

### Regras

Workflow é versionado. Cada run referencia uma versão imutável.

Uma definição deve conter, quando aplicável:

- ID/version;
- scope;
- steps/graph;
- input schema;
- conditions;
- retries;
- failure strategy;
- approval semantics;
- outputs/artifacts.

Invariantes:

- editar workflow não altera run em andamento;
- retry é explícito por step;
- secret é reference, não literal na definição;
- scheduled run usa service identity com capabilities explícitas;
- retry nunca pula policy/approval.

## 26. `approvals`

### Responsabilidades

- requests;
- decisions;
- requirements;
- reviewers;
- expiration.

### Estados

```text
PENDING
├── APPROVED
├── REJECTED
├── EXPIRED
└── CANCELLED
```

### Invariantes críticas

- approval é vinculado a hash/snapshot de intent material;
- mudar target, action ou input material invalida approval anterior;
- requester não satisfaz separation-of-duties quando policy exigir outro reviewer;
- expiration é validada no backend;
- decision registra actor, timestamp e reason/comment quando exigido;
- aprovação não é execução: authorization/policy pode ser revalidada antes do dispatch.

## 27. `policies`

### Responsabilidades

- definitions;
- evaluation;
- scopes;
- rules;
- allowlists;
- denylists;
- enforcement.

### Resultado normalizado

```text
ALLOW
DENY
REQUIRE_APPROVAL
```

com reasons e IDs das regras que contribuíram para a decisão.

### Invariantes

- enforcement é backend-side;
- rule pode avaliar actor, action, target, project, environment, provider, risk e contexto;
- deny explícito deve prevalecer sobre default permissivo quando a semântica da policy assim definir;
- production-like environments podem exigir regras mais fortes;
- alteração de policy é auditada.

## 28. `nodes`

### Responsabilidades

- registration;
- identity;
- connections;
- capabilities;
- heartbeats;
- health;
- dispatch;
- upgrades.

### Connection model

Node mantém conexão outbound com runtime coordenador. O protocolo deve suportar:

- authenticated node identity;
- credentials/certificates rotacionáveis;
- heartbeat;
- reconnect com bounded exponential backoff + jitter;
- capability advertisement;
- dispatch ACK;
- correlação/multiplexação de streams;
- cancellation;
- backpressure;
- graceful drain;
- version compatibility.

### Health states

```text
REGISTERING
ONLINE
DEGRADED
OFFLINE
DRAINING
UPGRADING
REVOKED
```

### Invariantes

- revoked node não aceita novo dispatch;
- node só executa capability anunciada e autorizada;
- node ID é diferente de machine display name;
- heartbeat loss não implica que processo remoto morreu;
- upgrade deve ser autenticado/verificado.

## 29. `agents`

### Responsabilidades

- identities;
- sessions;
- capabilities;
- delegations;
- tool access;
- activity;
- membership de co-gestão no escopo organization/project (agent como colaborador de project, não só caller efêmero).

### Regras

- agent possui identidade própria;
- agent autorizado é co-gestor do project no escopo das capabilities delegadas;
- mutações de agent são first-class no modelo de domínio e no audit;
- delegation registra issuer, target, capability set e validade;
- agent action atribui agent e, quando aplicável, delegating principal;
- agent não pode auto-elevar permissões;
- MCP tool availability deriva de capabilities;
- operações perigosas podem exigir human approval.

## 30. `logs`

### Responsabilidades

- ingestion;
- streams;
- storage;
- indexing;
- search;
- retention;
- exports.

### Limite do produto

O Polter pode mostrar logs necessários para entender/executar recursos, nodes, processes e runs. Isso não significa replicar Datadog/Grafana.

Prioridades:

- logs de run/session;
- logs de node/process;
- provider log stream leve quando útil;
- deep links para observability avançada.

### Invariantes

- retention/size são limitados;
- streams usam backpressure;
- truncation é explícita;
- secrets conhecidos são redigidos quando possível;
- renderer não acumula stream infinitamente.

## 31. `observability`

### Responsabilidades

- metrics;
- traces;
- health;
- monitors;
- dashboards;
- external links.

A observabilidade do próprio Polter usa OpenTelemetry. Para observabilidade de sistemas de usuário, priorizar health/context/deep links antes de construir uma suíte completa.

## 32. `audit`

### Responsabilidades

- events;
- actors;
- trails;
- integrity;
- retention;
- exports.

### Evento mínimo

```text
AuditEvent
├── id
├── occurred_at
├── actor_type
├── actor_id
├── organization_id
├── project_id?
├── environment_id?
├── action
├── target_type?
├── target_id?
├── result
├── correlation_id
├── request_id?
├── policy_decision_id?
├── approval_request_id?
├── metadata_redacted
├── integrity_metadata / previous_hash?
└── source_runtime
```

### Regras

- append-oriented;
- não confundir diagnostic logs com audit trail;
- secrets plaintext nunca entram em audit;
- streams de alto valor deveriam ser tamper-evident, por exemplo hash chain/checkpoints assinados;
- não usar a palavra “immutable” se a infraestrutura não garante isso de verdade.

## 33. `knowledge`

### Responsabilidades

- documents;
- decisions;
- ownership;
- runbooks;
- relationships;
- indexing;
- search.

Knowledge pode relacionar-se a:

- project;
- environment;
- system;
- component;
- resource;
- integration;
- workflow;
- operation;
- referência externa.

Runbook pode apontar para operation executável, mas execução continua passando por authorization/policy/approval.

## 34. `synchronization`

### Responsabilidades

- local state;
- cloud state;
- changes;
- conflicts;
- checkpoints;
- offline queue.

### Regras

- sync mutation é idempotente ou versionada;
- offline write carrega metadata suficiente para conflito;
- conflitos significativos não são silenciosamente sobrescritos;
- provider-derived state e user-authored state têm merge policy diferente;
- checkpoint permite resume;
- stale/offline precisa ser visível.

### Política de conflito recomendada

- campo user-authored: explicit conflict resolution ou revision rule visível;
- provider-derived field: provider vence, annotation fica separada;
- layout position: last-writer-wins pode ser aceitável com version/history;
- audit/operation immutable records: append, nunca overwrite merge.

## 35. `notifications`

### Responsabilidades

- events;
- preferences;
- deliveries;
- channels.

Notificar eventos acionáveis, não cada mudança interna.

Exemplos:

- approval solicitado/decidido;
- workflow failed;
- node offline relevante;
- integration auth expirou;
- scheduled operation concluiu;
- security/policy event importante.

## 36. `billing`

### Responsabilidades

- accounts;
- plans;
- subscriptions;
- usage;
- entitlements.

### Regras

- feature gating consulta entitlement service/interface;
- não espalhar `if plan === "pro"` pelo produto;
- downgrade preserva dados e falha de pagamento não destrói projeto;
- billing events/webhooks precisam ser idempotentes;
- usage cobravel deve ser reconciliável.

---

# Parte IV — Providers, Plataforma e Execução Distribuída

## 37. Providers suportados pela arquitetura

```text
cloud
├── aws
├── gcp
├── azure
└── generic-vps

platforms
├── vercel
├── supabase
├── github
├── gitlab
└── cloudflare

observability
├── grafana
├── datadog
├── prometheus
└── opentelemetry

messaging
├── rabbitmq
├── nats
└── kafka

databases
├── postgres
├── mysql
├── mongodb
└── redis

infrastructure
├── docker
├── kubernetes
├── terraform
└── local

communication
├── email
├── slack
└── webhooks
```

A presença de um diretório define a arquitetura de extensão; não significa que todo provider precise estar production-ready no primeiro release.

## 38. Contrato de adapter

Cada provider novo deve ter:

1. metadata do provider;
2. estratégia de auth/credentials;
3. capability declaration;
4. mapeamento para `Resource` normalizado;
5. estratégia de full/incremental sync;
6. paginação;
7. tratamento de rate limit;
8. normalized error mapping;
9. deep links;
10. revoke/reconnect;
11. fixtures/mocks;
12. integration tests quando sandbox existir;
13. documentação de scopes mínimos;
14. revisão de segurança.

### 38.1 Port conceitual

```text
Provider
├── Metadata()
├── Capabilities()
├── ValidateConnection()
├── DiscoverResources(cursor, filters)
├── GetResource(id)
├── GetStatus(id)
├── OpenExternalURL(id)
├── Execute(action, target, input)       # optional capability
├── StreamLogs(target, options)          # optional capability
├── Plan(change)                         # optional capability
├── Apply(plan)                          # optional capability
└── Revoke()/Close()
```

O domínio consumidor define a interface efetiva; este bloco é apenas a semântica esperada.

### 38.2 SDKs

SDK oficial pode ser usado para reduzir erro e custo de manutenção. Tipos do SDK não atravessam a fronteira do adapter.

### 38.3 Erros normalizados

```text
UNAUTHENTICATED
PERMISSION_DENIED
NOT_FOUND
CONFLICT
RATE_LIMITED
TEMPORARILY_UNAVAILABLE
INVALID_REQUEST
PROVIDER_FAILURE
UNSUPPORTED_CAPABILITY
```

Mensagem raw do provider pode existir em diagnostic context redigido, não como contrato de UI.

## 39. Platform layer

A camada `backend/platform/` oferece infraestrutura genérica:

```text
database
├── sqlite
├── postgres
├── migrations
└── transactions

transport
├── connect-rpc
├── grpc
├── http
├── websocket
└── local-ipc

messaging
├── outbox
├── jobs
├── events
└── nats

runtime
├── processes
├── terminal
├── filesystem
├── networking
├── signals
└── sandbox

security
├── authentication
├── authorization
├── encryption
├── secrets
├── certificates
└── signing

observability
├── logging
├── metrics
├── tracing
└── profiling

resilience
├── retries
├── timeouts
├── backpressure
├── rate-limiting
├── circuit-breakers
└── idempotency
```

Platform não deve esconder regra específica de produto dentro de helper genérico.

---

# Parte V — Contracts, APIs e Persistência

## 40. Organização dos schemas

```text
schemas/
├── api/
│   ├── public/
│   ├── internal/
│   ├── local/
│   └── node/
├── events/
│   ├── domain/
│   ├── integration/
│   └── audit/
├── database/
│   ├── local/
│   └── cloud/
└── configuration/
    ├── projects/
    ├── providers/
    ├── nodes/
    ├── workflows/
    └── policies/
```

### 40.1 Significado das APIs

- `public`: contratos estáveis intencionalmente expostos;
- `internal`: comunicação interna não prometida a clientes;
- `local`: desktop/local core;
- `node`: protocolo de controle/execução de nodes.

Uma RPC útil para o desktop não vira `public` automaticamente.

## 41. Regras Protocol Buffers

- enums possuem valor `UNSPECIFIED`/unknown-safe;
- field numbers removidos nunca são reutilizados;
- timestamp é representado consistentemente;
- IDs possuem campo explícito;
- lista grande usa cursor estável;
- mutation perigosa aceita idempotency/request ID quando aplicável;
- stream carrega session/correlation ID;
- errors têm machine-readable code;
- breaking change exige version strategy;
- generated code não é editado manualmente.

## 42. Query, Command e Stream

Não é necessário adotar CQRS completo, mas os contracts devem diferenciar intenção:

- **Query:** leitura sem mutation de negócio;
- **Command:** pedido de mutation/operation;
- **Stream:** subscription de output/state.

## 43. Catálogo conceitual de services

| Domínio         | Queries                             | Commands                         | Streams                                 |
| --------------- | ----------------------------------- | -------------------------------- | --------------------------------------- |
| Identity        | current actor, sessions             | authenticate/revoke/rotate       | security/session quando necessário      |
| Organizations   | orgs, members, teams                | create/update/invite/remove      | membership changes                      |
| Projects        | projects, environments              | create/update/archive/manage env | project changes                         |
| Architecture    | systems/components/views/snapshots  | edit/save/snapshot               | architecture changes                    |
| Resources       | inventory/detail/status             | discover/refresh/annotate/link   | resource/status changes                 |
| Integrations    | connections/capabilities/sync state | connect/revoke/reconnect/sync    | connection/sync state                   |
| Provisioning    | definitions/plans/drift             | plan/apply/cancel/rollback       | execution progress                      |
| Operations      | requests/executions/results         | request/cancel/schedule          | run/output/progress                     |
| Workflows       | definitions/versions/runs           | create/version/start/cancel      | run/step changes                        |
| Approvals       | inbox/history                       | approve/reject/cancel            | approval state                          |
| Policies        | definitions/explanations            | create/update/enable/disable     | policy changes                          |
| Nodes           | registry/health/capabilities        | register/revoke/drain/upgrade    | node health                             |
| Agents          | identities/delegations/activity     | create/revoke/delegate           | activity quando necessário              |
| Logs            | search/query                        | export/retention admin           | log stream                              |
| Observability   | health/monitors/links               | refresh/configure refs           | monitor health                          |
| Audit           | trail/export state                  | request export                   | sem mutation stream genérica por padrão |
| Knowledge       | docs/decisions/runbooks/search      | create/update/archive            | index/document changes                  |
| Synchronization | status/conflicts/checkpoints        | sync/retry/resolve               | sync progress                           |
| Notifications   | preferences/history                 | update/read                      | deliveries                              |
| Billing         | subscription/entitlements/usage     | change plan/portal intent        | entitlement changes                     |

## 44. Requisitos de streaming

Toda stream longa deve definir:

- ID de session/stream;
- correlation ID;
- ordering/sequence quando necessário;
- heartbeat/keepalive;
- final state;
- reconnect/resume quando suportado;
- backpressure;
- limite de buffer;
- comportamento de authorization lifetime.

## 45. SQLite local

Deve armazenar, conforme necessidade:

- preferences;
- local workspace/project state;
- local resource discovery;
- sync checkpoints;
- offline queue;
- local operation/session metadata;
- cache de cloud state necessário para UX/offline.

SQLite usa migrations versionadas.

## 46. PostgreSQL cloud

Armazena shared durable state:

- organizations/memberships/teams;
- projects/environments;
- architecture;
- resources;
- integration metadata/secret refs;
- operations/workflows;
- approvals/policies;
- nodes;
- agents/delegations;
- knowledge;
- audit;
- notifications;
- billing/entitlements;
- outbox.

## 47. Multi-tenancy

Toda linha tenant-scoped precisa ser rastreável para organization/project por relação inequívoca.

Regra crítica: backend não confia em `organization_id` enviado pelo cliente como prova de acesso.

Cross-tenant access precisa ser testado explicitamente.

## 48. Blueprint lógico de dados cloud

### Identity/Organizations

```text
users
sessions
credentials / credential_references
service_identities

organizations
organization_memberships
teams
team_memberships
invitations
```

### Projects

```text
projects
workspaces
environments
project_ownership
```

### Architecture

```text
systems
architecture_components
architecture_connections
architecture_dependencies
architecture_boundaries
architecture_views
architecture_layouts
architecture_snapshots
architecture_snapshot_items
```

Semantic model e layout devem permanecer separados.

### Resources/Integrations

```text
provider_connections
provider_credentials_or_secret_refs
provider_capabilities
integration_sync_runs
integration_webhook_registrations

resources
resource_relationships
resource_status_history
resource_annotations
resource_external_refs
component_resource_links
```

Unique identity recomendada quando provider oferecer ID estável:

```text
(provider_connection_id, provider_resource_id)
```

### Operations/Provisioning

```text
operation_requests
operation_executions
operation_sessions
operation_results
operation_stream_metadata
operation_schedules

provisioning_definitions
provisioning_plans
provisioning_executions
provisioning_state
provisioning_drift_records
```

### Workflows

```text
workflow_definitions
workflow_versions
workflow_steps
workflow_runs
workflow_step_runs
workflow_artifacts
workflow_triggers
```

`workflow_run` aponta para a versão exata usada.

### Governance

```text
policies
policy_versions
policy_rules
policy_scopes

approval_requests
approval_requirements
approval_reviewer_sets
approval_decisions

agent_identities
agent_sessions
agent_delegations
agent_activity
```

`approval_requests` deve persistir o intent digest usado para validar a aprovação.

### Nodes

```text
nodes
node_identities
node_credentials_or_cert_refs
node_connections
node_capabilities
node_heartbeats
node_health_history
node_upgrade_history
```

Heartbeat de alta frequência pode ter retention/compaction separada.

### Knowledge/Audit

```text
knowledge_documents
knowledge_decisions
knowledge_runbooks
knowledge_relationships
knowledge_index_state

audit_events
audit_integrity_checkpoints
audit_exports
```

### Synchronization

```text
sync_checkpoints
sync_change_log
sync_conflicts
offline_mutation_receipts
```

Offline mutation carrega mutation ID estável.

### Notifications/Billing

```text
notification_preferences
notification_deliveries

billing_accounts
subscriptions
plans
entitlements
usage_records
billing_event_receipts
```

Billing event receipt deve ser unique/idempotent.

## 49. Columns e concorrência

Quando aplicável:

```text
id
organization_id / project_id
created_at
updated_at
revision/version
archived_at / deleted_at apenas quando lifecycle exigir
```

Usar optimistic concurrency quando concurrent edits podem quebrar invariantes, especialmente em:

- architecture semantic edits;
- policies;
- workflow definitions;
- approvals;
- ownership;
- sync conflict resolution.

Não adicionar soft delete em toda tabela por reflexo.

## 50. Migrations

- migration é artefato revisado;
- destructive migration precisa de data-migration strategy;
- app rollout deve ser schema-compatible durante deploy;
- binary deve detectar schema incompatível claramente;
- rollback deve considerar data loss, não apenas rollback de container.

---

# Parte VI — Segurança e Governança

## 51. Trust boundaries

```text
Inputs não confiáveis
├── Renderer
├── CLI input
├── MCP input
├── provider/webhook payload
└── node-reported data

Boundaries privilegiadas
├── Electron preload/main
├── polter-core
├── polter-server
├── polter-node
└── provider credentials

Enforcement autoritativo
├── authorization
├── policy engine
├── approval engine
└── execution adapters
```

Nenhum check client-side é suficiente sozinho.

## 52. Capability-based authorization

Capabilities sugeridas:

```text
organization.read
organization.manage
project.read
project.manage
architecture.read
architecture.write
resource.read
resource.discover
integration.connect
integration.manage
operation.request
operation.execute
workflow.manage
approval.review
policy.manage
node.manage
knowledge.read
knowledge.write
audit.read
agent.manage
billing.manage
```

Capabilities podem ser scoped por organization/project/environment.

Roles apenas agregam capabilities.

## 53. Pipeline de operação privilegiada

Toda command privilegiada passa por:

1. authentication;
2. coarse authorization;
3. input validation;
4. target resolution;
5. policy evaluation;
6. approval verification;
7. provider/node capability check;
8. execution;
9. audit.

O fluxo é o mesmo para UI, CLI e agent.

## 54. Secrets

- plaintext secret não vai para ordinary domain table;
- local secrets devem usar secure storage do SO quando aplicável;
- cloud secrets usam mecanismo dedicado/KMS-backed;
- frontend recebe reference, não secret, quando possível;
- logs/audit/telemetry redigem secrets;
- credentials de provider precisam de revoke/rotation;
- bootstrap credential de node deve ser short-lived/single-use quando possível.

## 55. Segurança do Node

Node deve usar transporte autenticado mutuamente ou mecanismo equivalente robusto com identity rotation/revocation.

Controls obrigatórios:

- outbound channel;
- validação da identidade do server;
- validação da identidade do node;
- rotation/revocation;
- capability allowlist;
- command validation;
- nenhuma primitive de `eval` genérica;
- update assinado/verificado;
- permissões locais restritas para keys/config.

## 56. Terminal e comandos

O sistema distingue:

- structured operations;
- interactive terminal;
- provider API actions.

Structured operation deve ser preferida para ações repetíveis/perigosas.

Terminal session precisa:

- actor;
- target;
- autorização explícita;
- start/end;
- server/node-enforced close;
- correlation/audit;
- secrets ocultos do renderer;
- session recording somente se privacy/retention forem definidos explicitamente.

## 57. Auditabilidade mínima

Devem gerar audit event:

- login/session security changes;
- integration connect/revoke;
- membership/agent permission changes;
- policy changes;
- approval request/decision;
- node registration/revocation;
- operation/workflow start/result/cancel;
- destructive resource/provisioning action;
- credential rotation metadata;
- audit export.

---

# Parte VII — Frontend e UX

## 58. Direção visual

O Polter deve manter uma interface desktop minimalista, inspirada em ferramentas profissionais como Linear no tratamento de densidade e shell, sem copiar identidade visual.

Princípios definidos no projeto:

- sidebar integrada ao background;
- conteúdo principal em surface central discreta;
- pouco chrome;
- borders e spacing sutis;
- cards compactos;
- cards de architecture podem usar gradient sutil;
- edges minimalistas;
- badge central em edge quando a relação precisar de identidade/status;
- animação curta e funcional;
- cor de estado nunca é a única forma de distinção.

## 59. Design system

```text
packages/frontend/design-system/
├── tokens/
├── primitives/
├── components/
├── patterns/
├── icons/
└── themes/
```

Regras:

- HTML semântico primeiro;
- Ark UI para primitives complexas;
- Corvu para resize/panel;
- components do Polter encapsulam primitive + styling;
- feature não cria seu próprio Dialog/Select/Menu se o DS já possui;
- tokens são centralizados;
- provider icons passam por abstraction compartilhada.

## 60. Acessibilidade

UI production-ready deve suportar:

- keyboard navigation;
- focus visível;
- labels semânticas;
- correct focus management em dialogs/popovers/menus;
- reduced motion;
- contraste adequado;
- estado não dependente somente de cor;
- screen-reader semantics quando relevante.

## 61. Estado frontend

Separar:

- **server/domain state:** vindo de API/RPC;
- **application state:** session, org/project/environment atual;
- **view state:** selection, active tab/filter/panel;
- **preferences:** user settings persistidas;
- **stream state:** buffers temporários de terminal/log/run.

Não duplicar canonical domain data em múltiplos stores independentes.

## 62. Canvas

```text
canvas/
├── engine/
├── viewport/
├── graph/
├── nodes/
├── edges/
├── layouts/
├── interactions/
├── selection/
├── history/
└── rendering/
```

Regras:

- graph semantics fora de component rendering;
- layout separado de metadata;
- selection é view state;
- undo/redo do editor não é audit history;
- layout pesado roda em Worker;
- keys usam domain IDs estáveis;
- edge creation valida endpoints/tipos quando domínio exigir.

## 63. Logs UI

Obrigatório:

- virtualization;
- incremental append;
- pause/follow;
- filters;
- search;
- stdout/stderr/source quando disponível;
- truncation explícita;
- bounded memory.

## 64. Terminal UI

Renderer cuida da emulação/interação, não da execução.

API tipada esperada:

```text
CreateTerminalSession
WriteTerminalInput
ResizeTerminalSession
SignalTerminalSession
CloseTerminalSession
SubscribeTerminalOutput
```

Proibido expor `exec(commandString)` genérico pelo preload.

## 65. Estados obrigatórios de tela

Toda route/feature deve tratar:

- loading;
- empty;
- permission denied;
- offline/stale;
- partial data;
- recoverable error;
- fatal error;
- retry/reconnect;
- unavailable capability.

Mocks de UI não substituem esses estados na implementação real.

---

# Parte VIII — Arquitetura de Pastas

## 66. Árvore canônica do repositório

A árvore abaixo vem diretamente do documento-base de organização do Polter e deve ser preservada como taxonomia-alvo. Mudanças estruturais relevantes exigem ADR.

```text
polter/
│
├── apps/
│   ├── desktop/
│   │   ├── main/
│   │   │   ├── lifecycle/
│   │   │   ├── windows/
│   │   │   ├── menus/
│   │   │   ├── permissions/
│   │   │   ├── native/
│   │   │   ├── backend-supervisor/
│   │   │   └── updates/
│   │   ├── preload/
│   │   │   ├── bridge/
│   │   │   ├── channels/
│   │   │   └── security/
│   │   ├── renderer/
│   │   │   ├── bootstrap/
│   │   │   ├── shell/
│   │   │   ├── routes/
│   │   │   ├── desktop-features/
│   │   │   ├── desktop-adapters/
│   │   │   └── workers/
│   │   ├── resources/
│   │   ├── packaging/
│   │   └── distribution/
│   │
│   ├── web/
│   │   ├── bootstrap/
│   │   ├── shell/
│   │   ├── routes/
│   │   ├── web-features/
│   │   ├── web-adapters/
│   │   ├── workers/
│   │   └── public/
│   │
│   ├── cli/
│   │   ├── commands/
│   │   ├── interactive/
│   │   ├── output/
│   │   ├── authentication/
│   │   └── configuration/
│   │
│   ├── mcp/
│   │   ├── tools/
│   │   ├── resources/
│   │   ├── prompts/
│   │   ├── sessions/
│   │   ├── authorization/
│   │   └── transport/
│   │
│   └── website/
│       ├── marketing/
│       ├── documentation/
│       ├── downloads/
│       └── legal/
│
├── packages/
│   ├── frontend/
│   │   ├── application/
│   │   │   ├── composition/
│   │   │   ├── navigation/
│   │   │   ├── session/
│   │   │   ├── workspace/
│   │   │   └── routing/
│   │   ├── design-system/
│   │   │   ├── tokens/
│   │   │   ├── primitives/
│   │   │   ├── components/
│   │   │   ├── patterns/
│   │   │   ├── icons/
│   │   │   └── themes/
│   │   ├── platform/
│   │   │   ├── browser/
│   │   │   ├── desktop/
│   │   │   ├── storage/
│   │   │   ├── notifications/
│   │   │   ├── clipboard/
│   │   │   ├── filesystem/
│   │   │   └── process/
│   │   ├── data/
│   │   │   ├── api-client/
│   │   │   ├── rpc-client/
│   │   │   ├── cache/
│   │   │   ├── synchronization/
│   │   │   ├── subscriptions/
│   │   │   └── persistence/
│   │   ├── state/
│   │   │   ├── application/
│   │   │   ├── workspace/
│   │   │   ├── session/
│   │   │   └── preferences/
│   │   ├── canvas/
│   │   │   ├── engine/
│   │   │   ├── viewport/
│   │   │   ├── graph/
│   │   │   ├── nodes/
│   │   │   ├── edges/
│   │   │   ├── layouts/
│   │   │   ├── interactions/
│   │   │   ├── selection/
│   │   │   ├── history/
│   │   │   └── rendering/
│   │   ├── terminal/
│   │   │   ├── emulator/
│   │   │   ├── sessions/
│   │   │   ├── streams/
│   │   │   ├── commands/
│   │   │   └── themes/
│   │   ├── logs/
│   │   │   ├── viewer/
│   │   │   ├── streams/
│   │   │   ├── filters/
│   │   │   ├── search/
│   │   │   └── virtualization/
│   │   └── shared/
│   │       ├── utilities/
│   │       ├── formatting/
│   │       ├── validation/
│   │       ├── errors/
│   │       ├── telemetry/
│   │       └── testing/
│   │
│   ├── features/
│   │   ├── authentication/
│   │   ├── onboarding/
│   │   ├── organizations/
│   │   ├── workspaces/
│   │   ├── projects/
│   │   ├── environments/
│   │   ├── architecture/
│   │   ├── resources/
│   │   ├── integrations/
│   │   ├── provisioning/
│   │   ├── operations/
│   │   ├── workflows/
│   │   ├── terminal/
│   │   ├── logs/
│   │   ├── observability/
│   │   ├── nodes/
│   │   ├── members/
│   │   ├── agents/
│   │   ├── approvals/
│   │   ├── policies/
│   │   ├── audit/
│   │   ├── knowledge/
│   │   ├── notifications/
│   │   ├── search/
│   │   ├── command-palette/
│   │   ├── settings/
│   │   └── billing/
│   │
│   ├── contracts/
│   │   ├── common/
│   │   ├── identity/
│   │   ├── organizations/
│   │   ├── projects/
│   │   ├── architecture/
│   │   ├── resources/
│   │   ├── integrations/
│   │   ├── provisioning/
│   │   ├── operations/
│   │   ├── workflows/
│   │   ├── nodes/
│   │   ├── approvals/
│   │   ├── policies/
│   │   ├── audit/
│   │   ├── knowledge/
│   │   ├── synchronization/
│   │   └── events/
│   │
│   ├── generated/
│   │   ├── typescript/
│   │   └── go/
│   │
│   ├── configuration/
│   │   ├── typescript/
│   │   ├── linting/
│   │   ├── formatting/
│   │   ├── tailwind/
│   │   ├── vite/
│   │   ├── electron/
│   │   └── testing/
│   │
│   └── testing/
│       ├── fixtures/
│       ├── mocks/
│       ├── factories/
│       ├── integration/
│       └── end-to-end/
│
├── backend/
│   ├── runtimes/
│   │   ├── core/
│   │   ├── server/
│   │   ├── node/
│   │   ├── cli/
│   │   └── mcp/
│   │
│   ├── domains/
│   │   ├── identity/
│   │   │   ├── users/
│   │   │   ├── sessions/
│   │   │   ├── credentials/
│   │   │   ├── service-identities/
│   │   │   └── authentication/
│   │   ├── organizations/
│   │   │   ├── organizations/
│   │   │   ├── memberships/
│   │   │   ├── teams/
│   │   │   └── invitations/
│   │   ├── projects/
│   │   │   ├── projects/
│   │   │   ├── workspaces/
│   │   │   ├── environments/
│   │   │   └── ownership/
│   │   ├── architecture/
│   │   │   ├── systems/
│   │   │   ├── components/
│   │   │   ├── connections/
│   │   │   ├── dependencies/
│   │   │   ├── boundaries/
│   │   │   ├── views/
│   │   │   ├── layouts/
│   │   │   └── snapshots/
│   │   ├── resources/
│   │   │   ├── resources/
│   │   │   ├── resource-types/
│   │   │   ├── relationships/
│   │   │   ├── inventory/
│   │   │   ├── discovery/
│   │   │   ├── status/
│   │   │   └── metadata/
│   │   ├── integrations/
│   │   │   ├── connections/
│   │   │   ├── providers/
│   │   │   ├── credentials/
│   │   │   ├── capabilities/
│   │   │   ├── synchronization/
│   │   │   └── webhooks/
│   │   ├── provisioning/
│   │   │   ├── definitions/
│   │   │   ├── plans/
│   │   │   ├── executions/
│   │   │   ├── state/
│   │   │   ├── drift/
│   │   │   └── rollback/
│   │   ├── operations/
│   │   │   ├── commands/
│   │   │   ├── executions/
│   │   │   ├── sessions/
│   │   │   ├── streams/
│   │   │   ├── results/
│   │   │   ├── cancellation/
│   │   │   └── scheduling/
│   │   ├── workflows/
│   │   │   ├── definitions/
│   │   │   ├── steps/
│   │   │   ├── runs/
│   │   │   ├── triggers/
│   │   │   ├── conditions/
│   │   │   ├── retries/
│   │   │   └── artifacts/
│   │   ├── approvals/
│   │   │   ├── requests/
│   │   │   ├── decisions/
│   │   │   ├── requirements/
│   │   │   ├── reviewers/
│   │   │   └── expiration/
│   │   ├── policies/
│   │   │   ├── definitions/
│   │   │   ├── evaluation/
│   │   │   ├── scopes/
│   │   │   ├── rules/
│   │   │   ├── allowlists/
│   │   │   ├── denylists/
│   │   │   └── enforcement/
│   │   ├── nodes/
│   │   │   ├── registration/
│   │   │   ├── identity/
│   │   │   ├── connections/
│   │   │   ├── capabilities/
│   │   │   ├── heartbeats/
│   │   │   ├── health/
│   │   │   ├── dispatch/
│   │   │   └── upgrades/
│   │   ├── agents/
│   │   │   ├── identities/
│   │   │   ├── sessions/
│   │   │   ├── capabilities/
│   │   │   ├── delegations/
│   │   │   ├── tool-access/
│   │   │   └── activity/
│   │   ├── logs/
│   │   │   ├── ingestion/
│   │   │   ├── streams/
│   │   │   ├── storage/
│   │   │   ├── indexing/
│   │   │   ├── search/
│   │   │   ├── retention/
│   │   │   └── exports/
│   │   ├── observability/
│   │   │   ├── metrics/
│   │   │   ├── traces/
│   │   │   ├── health/
│   │   │   ├── monitors/
│   │   │   ├── dashboards/
│   │   │   └── external-links/
│   │   ├── audit/
│   │   │   ├── events/
│   │   │   ├── actors/
│   │   │   ├── trails/
│   │   │   ├── integrity/
│   │   │   ├── retention/
│   │   │   └── exports/
│   │   ├── knowledge/
│   │   │   ├── documents/
│   │   │   ├── decisions/
│   │   │   ├── ownership/
│   │   │   ├── runbooks/
│   │   │   ├── relationships/
│   │   │   ├── indexing/
│   │   │   └── search/
│   │   ├── synchronization/
│   │   │   ├── local-state/
│   │   │   ├── cloud-state/
│   │   │   ├── changes/
│   │   │   ├── conflicts/
│   │   │   ├── checkpoints/
│   │   │   └── offline-queue/
│   │   ├── notifications/
│   │   │   ├── events/
│   │   │   ├── preferences/
│   │   │   ├── deliveries/
│   │   │   └── channels/
│   │   └── billing/
│   │       ├── accounts/
│   │       ├── plans/
│   │       ├── subscriptions/
│   │       ├── usage/
│   │       └── entitlements/
│   │
│   ├── providers/
│   │   ├── cloud/
│   │   │   ├── aws/
│   │   │   ├── gcp/
│   │   │   ├── azure/
│   │   │   └── generic-vps/
│   │   ├── platforms/
│   │   │   ├── vercel/
│   │   │   ├── supabase/
│   │   │   ├── github/
│   │   │   ├── gitlab/
│   │   │   └── cloudflare/
│   │   ├── observability/
│   │   │   ├── grafana/
│   │   │   ├── datadog/
│   │   │   ├── prometheus/
│   │   │   └── opentelemetry/
│   │   ├── messaging/
│   │   │   ├── rabbitmq/
│   │   │   ├── nats/
│   │   │   └── kafka/
│   │   ├── databases/
│   │   │   ├── postgres/
│   │   │   ├── mysql/
│   │   │   ├── mongodb/
│   │   │   └── redis/
│   │   ├── infrastructure/
│   │   │   ├── docker/
│   │   │   ├── kubernetes/
│   │   │   ├── terraform/
│   │   │   └── local/
│   │   └── communication/
│   │       ├── email/
│   │       ├── slack/
│   │       └── webhooks/
│   │
│   ├── platform/
│   │   ├── database/
│   │   │   ├── sqlite/
│   │   │   ├── postgres/
│   │   │   ├── migrations/
│   │   │   └── transactions/
│   │   ├── transport/
│   │   │   ├── connect-rpc/
│   │   │   ├── grpc/
│   │   │   ├── http/
│   │   │   ├── websocket/
│   │   │   └── local-ipc/
│   │   ├── messaging/
│   │   │   ├── outbox/
│   │   │   ├── jobs/
│   │   │   ├── events/
│   │   │   └── nats/
│   │   ├── runtime/
│   │   │   ├── processes/
│   │   │   ├── terminal/
│   │   │   ├── filesystem/
│   │   │   ├── networking/
│   │   │   ├── signals/
│   │   │   └── sandbox/
│   │   ├── security/
│   │   │   ├── authentication/
│   │   │   ├── authorization/
│   │   │   ├── encryption/
│   │   │   ├── secrets/
│   │   │   ├── certificates/
│   │   │   └── signing/
│   │   ├── observability/
│   │   │   ├── logging/
│   │   │   ├── metrics/
│   │   │   ├── tracing/
│   │   │   └── profiling/
│   │   └── resilience/
│   │       ├── retries/
│   │       ├── timeouts/
│   │       ├── backpressure/
│   │       ├── rate-limiting/
│   │       ├── circuit-breakers/
│   │       └── idempotency/
│   │
│   └── compositions/
│       ├── local-core/
│       ├── cloud-server/
│       ├── remote-node/
│       ├── command-line/
│       └── mcp-server/
│
├── schemas/
│   ├── api/
│   │   ├── public/
│   │   ├── internal/
│   │   ├── local/
│   │   └── node/
│   ├── events/
│   │   ├── domain/
│   │   ├── integration/
│   │   └── audit/
│   ├── database/
│   │   ├── local/
│   │   └── cloud/
│   └── configuration/
│       ├── projects/
│       ├── providers/
│       ├── nodes/
│       ├── workflows/
│       └── policies/
│
├── infrastructure/
│   ├── development/
│   │   ├── local-environment/
│   │   ├── databases/
│   │   ├── messaging/
│   │   └── observability/
│   ├── cloud/
│   │   ├── networking/
│   │   ├── compute/
│   │   ├── database/
│   │   ├── storage/
│   │   ├── messaging/
│   │   ├── secrets/
│   │   └── observability/
│   ├── deployment/
│   │   ├── server/
│   │   ├── web/
│   │   ├── desktop/
│   │   ├── node/
│   │   ├── cli/
│   │   └── mcp/
│   ├── distribution/
│   │   ├── desktop-installers/
│   │   ├── node-installers/
│   │   ├── cli-releases/
│   │   └── container-images/
│   └── environments/
│       ├── development/
│       ├── staging/
│       └── production/
│
├── tooling/
│   ├── workspace/
│   ├── code-generation/
│   ├── migrations/
│   ├── releases/
│   ├── packaging/
│   ├── development/
│   ├── quality/
│   └── automation/
│
├── tests/
│   ├── architecture/
│   ├── contracts/
│   ├── domains/
│   ├── integrations/
│   ├── desktop/
│   ├── web/
│   ├── core/
│   ├── server/
│   ├── node/
│   ├── cli/
│   ├── mcp/
│   ├── security/
│   ├── performance/
│   ├── resilience/
│   └── end-to-end/
│
└── docs/
    ├── product/
    ├── architecture/
    ├── domains/
    ├── decisions/
    ├── protocols/
    ├── integrations/
    ├── security/
    ├── operations/
    ├── development/
    └── releases/
```

## 67. Regra para estrutura interna de domínio

Os documentos-base definem os bounded contexts, mas não exigem boilerplate idêntico em todo domínio. Para domínios não triviais, usar como referência:

```text
backend/domains/<domain>/
├── model/            # entities, value objects e invariantes
├── application/      # use cases, commands e queries
├── ports/            # interfaces de repository/provider/event
├── events/           # domain events
├── services/         # domain services
├── errors/           # erros estáveis do domínio
└── testing/          # helpers específicos quando necessário
```

Pastas vazias só para “parecer arquitetura” são proibidas. Estrutura deve acompanhar complexidade real.

## 68. Estrutura de feature frontend

Referência:

```text
packages/features/<feature>/
├── components/
├── routes/
├── data/
├── state/
├── actions/
├── forms/
├── models/
├── adapters/
├── testing/
└── index.ts
```

Regras:

- feature usa `packages/frontend` + contracts;
- feature não importa internals privados de outra feature;
- reusable UI sobe para design system;
- comportamento cross-feature sobe para application/data layer apropriada;
- route component fica fino.

## 69. Public boundaries

Cada package deve expor entrypoint público intencional. Evitar deep import de internals de outro package.

O monorepo não pode virar um monólito acoplado apenas dividido fisicamente em pastas.

## 70. Architecture tests

CI deveria bloquear imports proibidos, incluindo:

```text
backend/domains/**      -> backend/providers/**
backend/domains/**      -> apps/**
packages/design-system  -> features/**
renderer                -> fs/process/child_process/net privilegiados
```

---

# Parte IX — Regras de Engenharia

## 71. Tipagem e validação

### TypeScript

- strict mode;
- evitar `any` fora de boundary estreita e justificada;
- validar external/untrusted payload antes de entrar em aplicação;
- generated protobuf types não são redeclarados manualmente;
- IDs de domínios diferentes não devem ser confundidos por conveniência.

### Go

- errors internos mapeiam para stable error codes no boundary;
- context/deadlines devem ser propagados;
- repository/domain APIs representam necessidade do domínio, não SQL genérico;
- evitar global mutable state para correctness.

## 72. Tempo e IDs

- persisted timestamps em UTC;
- UI localiza apenas na presentation layer;
- IDs devem ser globalmente únicos;
- ordering precisa de critérios explícitos e deterministic tie-breaker;
- arrival order de stream não é automaticamente business order.

## 73. Idempotência

Avaliar e implementar para:

- provisioning;
- provider mutations;
- workflow start;
- operation request;
- approval decision submission;
- node dispatch;
- webhook processing;
- sync replay;
- billing events.

Idempotency key precisa de scope correto para não colidir comandos diferentes.

## 74. Concorrência

Aplicar optimistic concurrency ou lock explícito quando concurrent writes podem quebrar regra:

- architecture edits;
- policies;
- workflow definitions;
- approvals;
- ownership;
- credential rotation;
- conflict resolution.

Cliente recebe `CONFLICT`; nunca overwrite silencioso de estado novo por estado velho.

## 75. Archive, soft delete e hard delete

Usar semanticamente:

- `archive`: inativo, preservado e visível no lifecycle;
- `soft delete`: remoção lógica por necessidade operacional;
- `hard delete`: remoção física autorizada após retention/requirements.

Audit/execution history não deve desaparecer por cascade casual.

## 76. Error taxonomy

```text
INVALID_ARGUMENT
UNAUTHENTICATED
PERMISSION_DENIED
NOT_FOUND
ALREADY_EXISTS
CONFLICT
PRECONDITION_FAILED
APPROVAL_REQUIRED
POLICY_DENIED
UNSUPPORTED_CAPABILITY
RATE_LIMITED
CANCELLED
DEADLINE_EXCEEDED
OFFLINE
TEMPORARILY_UNAVAILABLE
PROVIDER_FAILURE
INTERNAL
```

Frontend nunca deve depender de string parsing de erro humano.

## 77. Timeouts

Toda operação externa possui deadline strategy.

- provider call: timeout apropriado ao provider;
- database: context deadline;
- node dispatch: ACK deadline + execution semantics;
- streams: heartbeat/idle semantics;
- request sem timeout ou cancelability precisa ser exceção consciente.

## 78. Retries

- apenas transient failures;
- bounded exponential backoff + jitter;
- respeitar `Retry-After`/rate-limit hint;
- nunca retry cego em mutation não idempotente;
- não retry validation/permission denied;
- exhaustion vira erro explícito.

## 79. Circuit breaking/degradação

Falha de um provider não deve derrubar o Polter inteiro.

Estado esperado:

- integration `DEGRADED`/`DISCONNECTED`;
- cache anterior acessível com stale marker;
- retries limitados;
- outras features continuam operacionais.

## 80. Backpressure

Obrigatório em:

- node streams;
- terminal;
- logs;
- websocket/RPC streams;
- event consumers;
- sync grande.

Drop/sample/truncate/block precisa ser política explícita.

---

# Parte X — Observabilidade e Performance

## 81. OpenTelemetry

Todos os runtimes production deveriam emitir telemetry correlacionável com:

- service/runtime;
- version/build;
- environment;
- request/correlation ID;
- domain operation;
- provider;
- result/error code;
- node ID quando relevante.

Tenant IDs só entram quando privacy/telemetry policy permitir.

Secrets nunca são telemetry attributes.

## 82. Logs estruturados

Campos recomendados:

```text
timestamp
level
service/runtime
message
event/error code
correlation_id
request_id?
actor_id?     # quando permitido
project_id?   # quando permitido
operation_id?
node_id?
provider?
```

Development pretty logging pode ser renderização do mesmo event estruturado.

## 83. Health/readiness

Distinguir:

- process alive;
- database reachable;
- schema compatible;
- required dependencies ready;
- optional integration degraded;
- runtime draining.

Process estar vivo não significa `ready`.

## 84. Performance como requisito

O Polter é desktop-first e precisa parecer rápido.

### Renderer

- evitar long tasks >50 ms nas interações normais;
- drag/canvas/resize devem manter frame delivery estável em hardware suportado;
- listas grandes virtualizam;
- graph layout/diff pesado vai para Worker;
- stream buffers têm limite;
- route transition com cache não deve causar rerender completo desnecessário.

### Backend

- publicar p50/p95/p99 para paths relevantes;
- evitar N+1 SQL/provider;
- queries de tabelas grandes devem ter index/query-plan revisado;
- provider sync pagina e checkpointa;
- local reads não bloqueiam em refresh externo desnecessário.

### Startup desktop

Ordem de prioridade:

1. window/shell;
2. `polter-core` ready;
3. last-known data;
4. reconciliação cloud/provider em background.

Nunca bloquear first useful render esperando refresh de todas integrations.

## 85. Limites obrigatórios

Definir limites para:

- log buffer;
- terminal scrollback;
- RPC/message payload;
- upload/artifact;
- concurrent sessions;
- concurrent operations;
- provider sync concurrency;
- event retries/dead letter;
- websocket fan-out;
- local cache retention.

---

# Parte XI — Testes e Quality Gates

## 86. Tipos de testes previstos na arquitetura

O documento-base reserva explicitamente:

```text
tests/
├── architecture/
├── contracts/
├── domains/
├── integrations/
├── desktop/
├── web/
├── core/
├── server/
├── node/
├── cli/
├── mcp/
├── security/
├── performance/
├── resilience/
└── end-to-end/
```

Essas categorias são parte da qualidade esperada.

## 87. Unit/domain tests

Cobrir:

- invariantes;
- state machines;
- policy evaluation;
- approval validity;
- workflow transitions;
- sync conflict logic;
- normalization;
- pure mappings/parsers.

## 88. Contract tests

Verificar:

- protobuf compatibility;
- generated client/server;
- stable error mapping;
- provider capability behavior;
- node protocol.

## 89. Integration tests

Exercitar:

- SQLite repositories;
- PostgreSQL repositories;
- outbox;
- ConnectRPC services;
- sync local↔cloud;
- node dispatch;
- provider adapters com fixtures/sandbox.

## 90. Desktop tests

Verificar:

- preload surface;
- channels proibidos;
- lifecycle de window;
- backend supervisor;
- routes críticas;
- keyboard/focus;
- updater/error flow.

## 91. Security tests

Obrigatório testar:

- authorization bypass;
- cross-tenant access;
- policy bypass;
- modified/replayed approval intent;
- expired/revoked sessions;
- revoked node;
- invalid/malicious IPC;
- secret redaction;
- webhook signature/auth;
- path/command injection boundary;
- MCP tool authorization.

## 92. Resilience tests

Simular:

- DB temporary failure;
- provider timeout/rate limit;
- node disconnect/reconnect;
- duplicate message;
- app restart durante sync queued;
- workflow retry/restart;
- expired integration auth;
- out-of-order stream event quando aplicável.

## 93. Performance tests

Cenários realistas:

- milhares de resources;
- canvas grande;
- long-running log stream;
- múltiplos nodes;
- alta frequência de stream updates;
- provider pagination;
- audit/history queries.

## 94. Determinismo de testes

Evitar dependência de clock/rede/conta externa não controlados.

Usar:

- injected clock;
- deterministic IDs quando útil;
- test DB/container;
- fixtures;
- fake transports;
- seeded data.

## 95. Mocks

Mocks são permitidos para desenvolvimento visual/testes isolados, mas:

- produção não usa mock adapter;
- mock fica em boundary de testing/dev;
- mock-only field não vira contract;
- telas devem ser testadas depois com latency, partial data, offline, error e permission states reais.

## 96. Coverage

Coverage percentual é métrica auxiliar.

- security-critical logic e state machines: branch coverage muito alta;
- core domains: coverage forte baseada em comportamento;
- generated/trivial wiring pode ser excluído;
- nenhum critical invariant fica “coberto” apenas por número agregado.

## 97. CI obrigatório

Conforme stack:

- TypeScript typecheck;
- lint;
- format check;
- Go format/vet/static analysis;
- dependency boundary checks;
- protobuf/schema compatibility;
- generated-code drift;
- unit/integration tests;
- security/dependency scanning;
- builds;
- packaging smoke tests.

## 98. Definition of Done

Feature não está pronta enquanto itens aplicáveis faltarem:

- domain behavior;
- backend authorization;
- policy/approval handling;
- contracts;
- migration;
- real data adapter;
- loading/empty/error/offline UI;
- accessibility;
- telemetry;
- audit;
- tests;
- error mapping;
- docs;
- architecture boundary compliance;
- production config;
- security review;
- E2E acceptance criteria.

---

# Parte XII — Deploy, Distribuição e Operação

## 99. Environments de infraestrutura

```text
development
staging
production
```

Secrets são separados. Production não herda local defaults implicitamente.

## 100. Artefatos de build

A arquitetura prevê distribuição de:

- desktop;
- web;
- server;
- node;
- CLI;
- MCP;
- container images;
- installers;
- website/docs.

Compatibilidade de protocolos entre versões suportadas precisa de teste.

## 101. Desktop release

Production release deve tratar:

- macOS signing/notarization;
- Windows code signing;
- Linux packaging quando suportado;
- auto-update assinado;
- update channels;
- rollback/recovery;
- compatibility entre desktop e local core;
- version diagnostics.

## 102. `polter-server`

Deve ser stateless fora de state externo explícito.

Requisitos:

- graceful shutdown;
- request draining;
- health/readiness;
- schema compatibility;
- horizontal scale sem correctness baseada em singleton memory;
- centralized secrets;
- PostgreSQL backup/restore;
- observable deployment version.

## 103. Node distribution

Installer de node deve:

- instalar service/binary claramente identificado;
- rodar com least privilege;
- proteger identity material;
- suportar uninstall/revoke;
- expor version/health;
- validar updates;
- não abrir inbound control port ampla por padrão.

## 104. Release flow

```text
change
  ↓
CI + architecture/security gates
  ↓
merge
  ↓
versioned build
  ↓
staging/internal channel
  ↓
smoke + migration + protocol verification
  ↓
progressive production release
  ↓
monitor
  ↓
promote / rollback
```

## 105. Backup/recovery

Antes de GA:

- automated PostgreSQL backups;
- restore testado periodicamente;
- retention definida;
- PITR se requisitos justificarem;
- RPO/RTO documentados e testáveis.

Não prometer disponibilidade/recovery que ainda não foi testado.

## 106. Incident readiness

Manter:

- service ownership;
- severity levels;
- rollback runbooks;
- security incident path;
- credential revoke procedure;
- node compromise/revoke procedure;
- provider outage behavior;
- customer communication path.

---

# Parte XIII — Regras de Negócio Transversais

## 107. Organizations

1. Todo project compartilhado pertence a uma organization.
2. Organization ativa deve manter ao menos um owner.
3. Organization-level policy pode restringir project-level policy.
4. Project não pode enfraquecer rule marcada como não sobrescrevível pela organization.
5. Remover member revoga novo acesso imediatamente; long-lived sessions precisam ser invalidadas/reautorizadas conforme session policy.
6. Archive/delete de organization preserva audit/billing records exigidos.

## 108. Projects

1. Project é o principal contêiner operacional do produto.
2. Project pode ser `ACTIVE` ou `ARCHIVED`.
3. Archived project é read-only salvo operações de restore/export/admin lifecycle.
4. Deletar project Polter NUNCA deleta recurso externo implicitamente.
5. Referências de architecture/resources/knowledge devem permanecer coerentes durante archive/delete lifecycle.

## 109. Environments

1. Environment pertence a um project.
2. Deletar environment não implica destruir recursos externos.
3. Operations podem ser scoped por environment.
4. Environment pode ser classificado como protected/production-like para policy.
5. Correspondência cross-environment é explícita; não inferir apenas por nome igual.

## 110. Architecture

1. Component pode existir sem resource conectado.
2. Resource pode existir no Inventory sem aparecer em Canvas.
3. Link component↔resource não transfere ownership do provider para Polter.
4. View/layout não duplica semantic component.
5. Snapshot é imutável.
6. Connection deve ter type/semantics pesquisáveis quando relevante.
7. Remover elemento de uma View não deve semanticamente deletar Component.
8. Semantic delete exige ação explícita e confirmação adequada.

## 111. Resources

1. Todo resource descoberto possui provenance.
2. Provider-owned fields são atualizados pelo provider.
3. User annotations ficam separadas quando provider refresh poderia sobrescrevê-las.
4. Desaparecimento temporário gera stale/missing lifecycle antes de delete quando provider não confirma deletion.
5. Actions dependem de capabilities.
6. Unsupported action fica indisponível com razão; nunca simular success.
7. Resource ID interno não deve depender só de display name.

## 112. Integrations

1. Connect registra scopes/capabilities.
2. Integration pode ser revogada.
3. Auth failure marca degraded/disconnected, mas não apaga last-known architecture.
4. Freshness/last sync ficam visíveis.
5. Rate limit aplica backoff.
6. Webhook é verificado quando provider oferece signature/secret.
7. Credentials nunca aparecem em logs normais.
8. Desconectar integration não destrói external resources.

## 113. Operations

1. Toda operation tem actor, action, target e correlation ID.
2. Validation ocorre antes de dispatch.
3. Authorization/policy no backend.
4. Approval vincula intent imutável.
5. Execution location é explícita: local core, node ou provider.
6. Resultado é explícito.
7. Cancellation é auditada.
8. Duplicate submission usa idempotência quando necessário.
9. Partial external state deve ser exposto honestamente.
10. Operation que não suporta rollback não mostra rollback falso.

## 114. Workflows

1. Definitions são versionadas.
2. Runs usam immutable version.
3. Inputs/outputs devem ser tipados quando razoável.
4. Failure strategy é explícita.
5. Approval gate pode estar no início ou por step.
6. Retry não bypassa policy.
7. Scheduled execution usa service identity.
8. Artifact/outputs não podem vazar secrets por default.

## 115. Approvals

1. `APPROVED` não significa “já executado”.
2. Approval expira conforme policy.
3. Material intent change invalida approval.
4. Rejection encerra request corrente.
5. Decision é fato histórico; correção cria novo event/request, não sobrescreve audit.
6. Reviewer eligibility é avaliada no momento da decisão.
7. Separation-of-duties deve ser enforce no backend.

## 116. Policies

1. Policy avalia actor + action + target + scope + context.
2. UI nunca é enforcement final.
3. Policy version deve ser rastreável para uma decision relevante.
4. Mudança de policy é auditada.
5. Protected environment pode exigir human approval mesmo para agent autorizado.

## 117. Nodes

1. Node possui identidade própria.
2. Machine hostname é metadata, não identity.
3. Node revogado não recebe dispatch.
4. Server não assume capability que node não anunciou.
5. Node reconnect é esperado e não cria nova identidade por acidente.
6. Heartbeat loss vira `OFFLINE`/`DEGRADED`; não marca running process como failed automaticamente.
7. Upgrade pode drenar node antes de aplicar update.

## 118. Agents

1. Agent possui actor ID distinto.
2. Session possui capabilities explícitas.
3. Tool list deriva do authorization context.
4. Agent não concede capability a si mesmo.
5. Agent-triggered mutation é identificável em audit.
6. Human approval continua obrigatório quando policy exigir.
7. MCP output minimiza sensitive context.
8. Agent change em architecture é diferente de provider-observed change e de human change.
9. Agent autorizado é co-gestor do project no escopo das capabilities: architecture, resources, knowledge, operations e access metadata compartilhados com humanos.
10. Superfícies humanas (Access · Activity · Audit · Inbox) DEVEM permitir administrar e inspecionar a colaboração humano+agent; MCP/CLI DEVEM executar as mesmas capabilities autorizadas.

## 119. Knowledge

1. Knowledge gerenciado pelo Polter é project/system context, não notebook genérico.
2. Relationships são tipadas.
3. Archive de objeto não destrói decisão histórica relacionada.
4. Runbook executável apenas referencia operation/workflow; não executa bypass.
5. Search indexing pode ser eventual, mas falha/staleness precisa de comportamento definido.

## 120. Audit

1. Append-oriented.
2. Audit e diagnostic logging são separados.
3. Actor/action/target/result usam semântica estável.
4. Secrets redigidos.
5. Export de audit é access-controlled e auditado.
6. Retention job também produz evidência administrativa.

## 121. Billing e Entitlements

1. Entitlement controla capability/limit/quota.
2. Billing webhook é idempotente.
3. Payment failure não destrói customer data.
4. Downgrade behavior é determinístico.
5. Usage usado para billing é reconciliável.
6. Feature não deve saber nome comercial do plano quando só precisa perguntar entitlement.

---

# Parte XIV — Contrato para Agentes de IA que Implementam o Polter

## 122. Regra geral

Todo coding agent deve tratar este documento como specification contract, não como inspiração opcional.

Antes de editar código, deve identificar:

1. qual feature está sendo implementada;
2. qual domínio é owner;
3. quais contracts existentes se aplicam;
4. quais runtimes serão afetados;
5. se há migration;
6. implicações de authorization/policy/approval;
7. implicações de audit;
8. error/offline states;
9. testes necessários;
10. impacto no design system e navegação.

## 123. Proibições arquiteturais para agents

Agent NÃO DEVE:

- colocar business logic em route component;
- colocar business logic em Electron Main;
- acessar DB diretamente do frontend;
- importar provider SDK diretamente em domain;
- expor bridge Electron genérica;
- criar segundo canonical store do mesmo aggregate;
- criar microservice sem ADR;
- bypassar policy/approval/audit para “fazer funcionar”;
- adicionar provider special-case em UI genérica quando capability metadata resolve;
- mudar hierarchy/sidebar silenciosamente;
- mover responsabilidade entre domains sem justificar;
- duplicar generated contracts manualmente.

## 124. Proibições de fake production behavior

Nunca shipar:

- mock data em production route;
- auth check placeholder;
- `TODO authorize later` em mutation privilegiada;
- success hard-coded;
- secret em source;
- TLS verification desabilitada;
- `catch {}` silencioso em operação crítica;
- unbounded retry;
- unbounded stream buffer;
- provider mutation sem audit;
- broad shell bridge no renderer.

## 125. Implementação vertical

Feature cross-layer deveria ser implementada nesta ordem lógica:

```text
contract
  ↓
domain behavior
  ↓
repository/provider ports
  ↓
implementation + migration
  ↓
runtime composition
  ↓
frontend data adapter
  ↓
feature UI
  ↓
tests
  ↓
telemetry + audit + docs
```

Não considerar uma tela “pronta” porque mock está visualmente bonito.

## 126. Comportamento não especificado

Se faltar regra material:

- escolher default mais seguro e reversível;
- não inventar destructive/security semantics silenciosamente;
- isolar decisão atrás de interface/configuração;
- documentar assumption;
- criar ADR se alterar arquitetura.

## 127. Refactor

Refactor deve preservar:

- contracts externos;
- domain invariants;
- authorization;
- policy/approval semantics;
- audit semantics;
- data compatibility;
- user behavior fora do escopo da mudança.

Refactor não é licença para feature creep.

## 128. Checklist por tipo de mudança

### UI-only

- design system primitives;
- keyboard/focus;
- loading/error/offline states;
- nenhum privileged direct access;
- responsive desktop behavior;
- visual regression quando disponível.

### Nova mutation

- contract;
- validation;
- authorization;
- policy;
- approval;
- idempotency;
- transaction;
- audit;
- integration tests.

### Novo provider

- capability map;
- scopes mínimos;
- resource normalization;
- pagination;
- rate limit/retry;
- errors;
- sync checkpoint;
- revoke;
- fixtures/tests;
- docs/deep links.

### Nova node capability

- protocol;
- advertise capability;
- authorization/policy;
- runtime/sandbox boundary;
- timeout/cancel;
- backpressure;
- compatibility;
- security tests;
- audit.

### Nova MCP tool

- owning domain;
- minimal schema;
- auth;
- capability gating;
- policy/approval;
- redaction;
- audit;
- safe errors;
- sem generic shell por padrão.

---

# Parte XV — Plano de Entrega até Production Ready

## 129. Fase 0 — Fundação do repositório

Entregáveis:

- monorepo canônico;
- TypeScript/Go config;
- lint/format;
- protobuf generation;
- Go runtimes skeleton;
- architecture tests;
- dev environment;
- CI;
- ADR process;
- security baseline.

Gate:

- clean clone builda;
- codegen é reproduzível;
- CI detecta boundary violation;
- desktop e runtimes Go buildam separadamente.

## 130. Fase 1 — Desktop shell + local core

Entregáveis:

- Electron lifecycle;
- secure preload;
- backend supervisor;
- SolidJS shell/routes;
- Ark UI design system;
- SQLite migrations;
- local RPC;
- project/environment context;
- telemetry/error baseline.

Gate:

- renderer sem Node privilege;
- core restart/start confiável;
- foundational routes usando dados reais.

## 131. Fase 2 — Architecture + Resources

Entregáveis:

- architecture domain;
- Canvas/Components/Views;
- resource domain;
- Inventory;
- local discovery foundation;
- component-resource linking;
- snapshots/history;
- search/filter;
- Worker para graph heavy work.

Gate:

- Canvas e Components usam mesmo model;
- inventory/graph grande continua responsivo;
- provenance/freshness visíveis.

## 132. Fase 3 — Cloud + Organizations + Sync

Entregáveis:

- server runtime;
- PostgreSQL;
- organizations/memberships/projects;
- auth/session foundation;
- local↔cloud sync;
- offline queue;
- conflicts;
- outbox;
- shared audit foundation.

Gate:

- dois clients colaboram sem silent corruption;
- cross-tenant tests passam;
- reconnect/offline path testado.

## 133. Fase 4 — Integrations

Entregáveis:

- provider framework;
- primeiros adapters;
- secret handling;
- capabilities;
- sync/webhooks;
- integration health UI;
- deep links;
- normalized resources.

Gate:

- provider outage/rate-limit não derruba app;
- expired auth degrada corretamente;
- zero credential leakage em telemetry/audit.

## 134. Fase 5 — Nodes + Operations

Entregáveis:

- node registration/identity;
- outbound connection;
- heartbeat/reconnect;
- capability negotiation;
- dispatch;
- operation lifecycle;
- streams;
- process/terminal runtime adapters;
- backpressure;
- cancellation;
- node health UI.

Gate:

- revoke path testado;
- duplicate dispatch controlado;
- bounded stream memory;
- offline node behavior explícito.

## 135. Fase 6 — Access + Policies + Approvals

Entregáveis:

- capability authorization;
- policies;
- approvals;
- Inbox;
- Members/Agents Access UI (People e Agents como co-managers do project);
- Activity/Audit com distinção de actor `person` | `agent` | `system`;
- separation-of-duties;
- approval intent binding;
- audit export controls.

Gate:

- UI/CLI/MCP não executam privileged operation sem mesma governança;
- approval replay/intent mutation attacks falham.

## 136. Fase 7 — Workflows + CLI + MCP

Entregáveis:

- workflow versioning/engine;
- retries/conditions/artifacts;
- scheduling foundation;
- CLI estável;
- MCP tools/resources cobrindo as mesmas co-management capabilities do project (não só operations perigosas);
- agent delegation;
- agent activity/audit.

Gate:

- workflow failure/retry determinístico;
- agent não se auto-eleva;
- CLI/MCP seguem domain semantics.

## 137. Fase 8 — Knowledge + Logs + Observability composition

Entregáveis:

- documents/decisions/runbooks;
- relationships;
- indexing/search;
- run/node log viewer;
- health/deep links de observability;
- notifications.

Gate:

- knowledge conectado a domain objects;
- logs bounded/virtualized;
- produto não replica suíte completa de observability.

## 138. Fase 9 — Production hardening

Entregáveis:

- threat model final;
- dependency/supply-chain review;
- signed releases;
- backup/restore drill;
- load/performance tests;
- stream soak tests;
- resilience tests;
- migration compatibility;
- staging production-like;
- support diagnostics;
- incident/runbooks;
- privacy/legal review aplicável.

---

# Parte XVI — Production Release Gates

## 139. Gate funcional

Antes de chamar o produto de production-ready:

- todas as primary routes usam real adapters;
- org/project/environment end-to-end;
- architecture/resources persistem corretamente;
- integration connect/sync/revoke/error;
- operation lifecycle correto;
- policy/approval enforce backend-side;
- node reconnect robusto;
- audit cobre security-significant actions;
- CLI/MCP não bypassam regras;
- customer data export/lifecycle necessário funciona.

## 140. Gate de segurança

- threat model revisado;
- nenhum critical/high exploitável aceito sem risk decision explícita;
- Electron boundary testado;
- tenant isolation testada;
- secret scanning/redaction validada;
- node identity/update verification testados;
- dependency vulnerabilities triadas;
- provider scopes documentados;
- approval/policy bypass tests verdes.

## 141. Gate de confiabilidade

- backup restore testado;
- reconnect soak testado;
- provider rate limit/timeout testado;
- outbox duplicate delivery testado;
- offline sync recovery testado;
- graceful shutdown testado;
- migration rollout testado;
- unknown execution outcome tratado.

## 142. Gate de performance

- canvas grande interativo;
- inventory grande responsivo;
- logs long-running bounded;
- hot SQL queries indexadas;
- provider sync paginado/checkpointado;
- startup sem bloqueio por refresh total;
- long-lived session memory soak sem leak relevante.

## 143. Gate de qualidade

- CI green em targets suportados;
- generated-code drift zero;
- architecture tests green;
- critical E2E green;
- artifacts assinados/empacotados;
- compatibility matrix documentada;
- docs/runbooks atualizados.

## 144. Gate operacional

- alerts para cloud critical paths;
- rollback exercitado;
- incident/security runbooks;
- support diagnostics;
- audit inspection/export;
- backup/recovery responsável definido;
- release ownership definido.

---

# Parte XVII — Decisões Arquiteturais Atuais

## 145. Tabela normativa

| Área                    | Decisão atual                                                               |
| ----------------------- | --------------------------------------------------------------------------- |
| Produto                 | Control plane que une arquitetura, infraestrutura, conhecimento e operações |
| Usuários                | Pessoas e agents de IA co-gerenciam projects sob a mesma governança         |
| Desktop                 | Electron                                                                    |
| Renderer                | SolidJS                                                                     |
| Styling                 | Tailwind CSS v4                                                             |
| Primitives acessíveis   | Ark UI, substituindo Kobalte                                                |
| Resizable panels        | Corvu                                                                       |
| Large lists/logs        | TanStack Virtual                                                            |
| Heavy frontend work     | Web Workers                                                                 |
| Backend                 | Go                                                                          |
| API                     | ConnectRPC + Protocol Buffers                                               |
| Local DB                | SQLite                                                                      |
| Cloud DB                | PostgreSQL                                                                  |
| PostgreSQL access       | pgx + sqlc                                                                  |
| Server architecture     | Monólito modular                                                            |
| Reliable initial events | PostgreSQL transactional outbox                                             |
| Scale event path        | NATS JetStream                                                              |
| Observability           | OpenTelemetry + structured logs                                             |
| Remote execution        | Polter Node outbound                                                        |
| Agent interface         | MCP                                                                         |
| Domain rule             | Domains não dependem de concrete providers/apps/Electron/web                |

## 146. Decisões que ainda exigem ADR quando concretizadas

Os dois documentos não fecham:

- cloud hosting vendor;
- exact identity provider;
- exact secret manager;
- exact node transport framing;
- exact canvas rendering library;
- object storage;
- billing provider;
- analytics provider;
- retention periods;
- pricing/plan quotas;
- exact SLO/RPO/RTO;
- supported OS/version matrix;
- final role matrix;
- exact terminal recording policy.

Agent não transforma library choice casual em decisão arquitetural permanente.

---

# Parte XVIII — Glossário do Projeto

## 147. Termos

| Termo         | Significado no Polter                                                                                               |
| ------------- | ------------------------------------------------------------------------------------------------------------------- |
| Control plane | Camada que coordena estado, governança e ações sobre sistemas sem necessariamente executar todo workload subjacente |
| System        | Raiz lógica de arquitetura dentro de um project                                                                     |
| Component     | Elemento lógico/tipado da arquitetura                                                                               |
| Resource      | Objeto real/normalizado de infraestrutura ou plataforma                                                             |
| Connection    | Relação/fluxo explícito entre componentes                                                                           |
| Dependency    | Dependência semântica entre elementos                                                                               |
| View          | Projeção/filtro salvo do modelo canônico                                                                            |
| Layout        | Estado visual de posicionamento de uma view                                                                         |
| Snapshot      | Captura imutável de architecture state                                                                              |
| Provider      | Adapter de sistema externo/local que implementa capabilities                                                        |
| Integration   | Conexão configurada/autenticada com provider                                                                        |
| Node          | Agente Polter de execução remota                                                                                    |
| Operation     | Pedido governado de ação                                                                                            |
| Run           | Instância de execução de operation/workflow                                                                         |
| Workflow      | Orquestração reutilizável e versionada                                                                              |
| Policy        | Regra que decide allow/deny/require approval                                                                        |
| Approval      | Decisão de governança vinculada a intent específico                                                                 |
| Agent         | Identidade de IA/automação que co-gerencia projects com humanos sob capabilities delegadas                          |
| MCP           | Superfície programática de recursos/tools para agents co-gerenciarem e operarem o project                           |
| Audit event   | Evidência estruturada de actor/action/target/result                                                                 |
| Knowledge     | Docs, decisões, ownership e runbooks ligados ao sistema                                                             |
| Capability    | Habilidade de leitura/ação exposta por auth/provider/node                                                           |
| Outbox        | Padrão que persiste event junto à transaction antes de publicação assíncrona                                        |
| Backpressure  | Controle para produtor não sobrecarregar consumidor de stream                                                       |
| Idempotency   | Repetição segura de request sem efeito duplicado indevido                                                           |

---

# Parte XIX — Contrato Final do Produto

## 148. Definição curta canônica

O Polter é um control plane desktop-first que transforma a arquitetura de software em um modelo vivo e operacional. O produto combina um workspace Electron/SolidJS com um backend local em Go, um backend cloud, integrações normalizadas, execução por Polter Nodes conectados outbound, operations/workflows governados por identities, capabilities, policies e approvals, auditabilidade e knowledge associado ao mesmo modelo arquitetural. Ele não substitui as plataformas especializadas; ele conecta, contextualiza e governa o uso delas.

## 149. Propriedades não negociáveis

Uma implementação só representa corretamente o Polter se:

1. houver **um modelo canônico** compartilhado por Canvas, listas, API e operations;
2. domains forem independentes de concrete providers e UI;
3. renderer não tiver direct OS privilege;
4. privileged actions forem autorizadas e policy-enforced backend-side;
5. humans e agents usarem a mesma governança;
6. humans e agents co-gerenciarem o mesmo project model com distinção de actor e sem path privilegiado para agent;
7. approvals estiverem vinculados a intent material imutável;
8. ações importantes forem auditáveis;
9. nodes conectarem outbound e executarem só capabilities autorizadas;
10. streams forem bounded e suportarem backpressure;
11. provider state for normalizado sem false ownership;
12. local/cloud sync tiver conflito/offline semantics explícitas;
13. contracts vierem de Protocol Buffers;
14. mocks não entrarem em release path;
15. security, resilience, performance e accessibility forem release criteria;
16. architecture boundaries forem enforce por CI/tests.

## 150. Regra para decisões futuras

Quando houver dúvida, preferir a solução que:

1. preserve domain isolation;
2. minimize privilege;
3. seja reversível;
4. torne estado/falha explícitos;
5. contenha provider complexity no adapter;
6. não bloqueie renderer;
7. seja auditável;
8. seja testável sem production credentials;
9. evite premature distributed complexity;
10. possa ser substituída atrás de interface sem reescrever business behavior.

---

# Apêndice A — Catálogo sugerido de Domain Events

```text
identity.user_created
identity.session_started
identity.session_revoked
identity.credential_rotated

organization.created
organization.member_added
organization.member_removed
organization.team_changed
organization.invitation_created

project.created
project.archived
project.environment_created
project.environment_updated

architecture.system_created
architecture.component_created
architecture.component_updated
architecture.connection_created
architecture.snapshot_created

resource.discovered
resource.updated
resource.status_changed
resource.missing
resource.deleted_external

integration.connected
integration.disconnected
integration.sync_started
integration.sync_completed
integration.sync_failed
integration.credential_expired

provisioning.plan_created
provisioning.execution_started
provisioning.execution_completed
provisioning.drift_detected

operation.requested
operation.approval_required
operation.dispatched
operation.started
operation.completed
operation.failed
operation.cancelled

workflow.definition_created
workflow.definition_versioned
workflow.run_started
workflow.step_started
workflow.step_completed
workflow.step_failed
workflow.run_completed
workflow.run_failed

approval.requested
approval.approved
approval.rejected
approval.expired

policy.created
policy.updated
policy.evaluated

node.registered
node.connected
node.disconnected
node.health_changed
node.revoked
node.upgrade_started
node.upgrade_completed

agent.created
agent.delegated
agent.session_started
agent.tool_invoked
agent.revoked

knowledge.document_created
knowledge.decision_recorded
knowledge.runbook_updated

sync.checkpoint_advanced
sync.conflict_detected
sync.conflict_resolved

notification.delivery_requested
notification.delivered
notification.failed

billing.subscription_changed
billing.entitlement_changed
```

Eventos só carregam payload necessário. Secret-bearing payload é proibido.

---

# Apêndice B — Classificação sugerida de risco de Operation

| Risco       | Exemplos                                        | Governança default                                                        |
| ----------- | ----------------------------------------------- | ------------------------------------------------------------------------- |
| `READ_ONLY` | inspect/list/status                             | authorization                                                             |
| `LOW`       | refresh sync, restart local dev process         | authorization + policy                                                    |
| `MEDIUM`    | shared non-prod restart/config change           | policy; approval configurável                                             |
| `HIGH`      | production restart/scaling/config mutation      | policy + approval por padrão                                              |
| `CRITICAL`  | destructive production/security/access mutation | explicit policy + strong approval/separation-of-duties; pode ser disabled |

Provider pode sugerir risk metadata, mas decisão final pertence à policy do Polter.

---

# Apêndice C — Mensagens conceituais do Node Protocol

```text
RegisterNode
RegisterNodeResponse
AuthenticateNode
NodeHello
NodeHeartbeat
NodeCapabilityUpdate
DispatchOperation
DispatchAck
OperationStarted
OperationOutputChunk
OperationProgress
OperationResult
CancelOperation
CancelAck
OpenTerminalSession
TerminalInput
TerminalOutput
TerminalResize
TerminalSignal
CloseTerminalSession
DrainNode
UpgradeNode
NodeGoodbye
```

Mensagens precisam de node/session/operation correlation e protocol/version metadata.

---

# Apêndice D — Documentação obrigatória do repositório

```text
docs/product/
├── product-definition.md
├── information-architecture.md
└── business-rules.md

docs/architecture/
├── system-overview.md
├── frontend.md
├── backend.md
├── data-flow.md
└── repository-boundaries.md

docs/domains/
└── <um documento por domínio>

docs/decisions/
└── ADR-xxxx-*.md

docs/protocols/
├── api-versioning.md
├── node-protocol.md
└── synchronization.md

docs/integrations/
└── <provider docs>

docs/security/
├── threat-model.md
├── authorization.md
├── secret-management.md
├── electron-security.md
└── node-security.md

docs/operations/
├── deployment.md
├── backup-restore.md
├── incident-response.md
└── release-rollback.md

docs/development/
├── setup.md
├── testing.md
├── code-generation.md
└── contributing.md

docs/releases/
└── compatibility-and-changelog.md
```

---

# Apêndice E — Checklist de Pull Request

## Produto

- Resolve um use case real do Polter?
- Está virando clone de ferramenta especializada sem necessidade?
- Está na feature/domínio correto?

## Arquitetura

- Quem é o owning domain?
- Dependency direction foi respeitada?
- Provider logic vazou para domain/UI?
- Há canonical state duplicado?

## Segurança

- Qual novo privilege foi criado?
- Backend autoriza de verdade?
- Policy/approval se aplicam?
- Secrets podem vazar?
- Audit existe?
- Agent consegue fazer algo que human equivalente não conseguiria?

## Reliability

- Timeout/retry/disconnect definidos?
- Operation é idempotente ou safely non-retried?
- Stream é bounded?
- Unknown/partial outcome é representado honestamente?

## Data

- Precisa migration?
- Tenant scoping correto?
- Concurrent writes corretos?
- Archive/delete semantics explícitas?

## Frontend

- Design system usado?
- Loading/error/offline/permission states completos?
- Renderer é bloqueado?
- Keyboard/focus/accessibility corretos?
- Visual language do Polter preservada?

## Testes

- Invariantes testadas?
- Integration boundary testada?
- Security failure path testado?
- E2E prova o comportamento do usuário?

## Operações

- Logs/traces/metrics permitem diagnóstico?
- Production config segura?
- Rollout/rollback compatíveis?

---

# Apêndice F — Notas de Reconciliação das Fontes

1. O documento de stack original usa Kobalte; decisão posterior do projeto substituiu Kobalte por Ark UI. Esta especificação considera Ark UI atual.
2. A árvore de repositório possui `web` e `desktop`. Isso define capacidade arquitetural; a direção atual do produto permanece desktop-first.
3. A existência de domínios `logs`, `observability`, `provisioning` e `billing` define boundaries e evolução possível, não obrigação de replicar integralmente produtos especialistas.
4. `members` e `terminal` aparecem na organização de features, enquanto no backend suas responsabilidades são compostas por `identity/organizations` e `operations/runtime`, evitando bounded contexts duplicados.
5. Os dois documentos-base não definem completamente role matrix, auth provider, hosting vendor, secret manager, SLOs, retention, billing provider, exact node handshake ou terminal recording policy. Essas decisões precisam ser formalizadas por ADR quando concretizadas.

---

**Fim da especificação.**
