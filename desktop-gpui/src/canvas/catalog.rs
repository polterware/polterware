//! What each canvas kind is (`components.ts`, `component-icons.ts`, `shapes.ts`, `types/`):
//! the component types the core allows (mirroring `canvas.ComponentKinds`), the icon each
//! one carries, the library's groups, and the generalist canvas type's behavior.
//!
//! The drawings and silhouettes themselves are polterware-ui's (`canvas::shapes`).

use polterware_ui::canvas::{FAMILIES, Family, SHAPES, ShapeGroup, family_of, shape_of};
use polterware_ui::icons::Icon;

use crate::contracts::CanvasTypeDefinition;

/// A component type: what it is, its family, and the technologies suggested for it.
#[derive(Clone, Copy, Debug)]
pub struct Component {
    pub kind: &'static str,
    pub label: &'static str,
    pub family: &'static str,
    pub technologies: &'static [&'static str],
    pub icon: Icon,
}

const fn component(
    kind: &'static str,
    label: &'static str,
    family: &'static str,
    technologies: &'static [&'static str],
    icon: Icon,
) -> Component {
    Component {
        kind,
        label,
        family,
        technologies,
        icon,
    }
}

/// `COMPONENTS`, in palette order.
pub const COMPONENTS: [Component; 42] = [
    component(
        "frontend",
        "Web app",
        "client",
        &[
            "React", "Solid", "Next.js", "Vue", "Svelte", "Angular", "Astro",
        ],
        Icon::Globe,
    ),
    component(
        "mobile",
        "Mobile app",
        "client",
        &["React Native", "Flutter", "Swift", "Kotlin", "Expo"],
        Icon::Smartphone,
    ),
    component(
        "desktop",
        "Desktop app",
        "client",
        &["Electron", "Tauri", "SwiftUI", "Qt"],
        Icon::Monitor,
    ),
    component(
        "cli",
        "CLI",
        "client",
        &["Go", "Rust", "Node.js", "Python"],
        Icon::SquareTerminal,
    ),
    component(
        "service",
        "Service",
        "compute",
        &["Go", "Node.js", "Python", "Rust", "Java", "Elixir", ".NET"],
        Icon::Server,
    ),
    component(
        "api",
        "API",
        "compute",
        &["REST", "GraphQL", "gRPC", "tRPC", "WebSocket"],
        Icon::Plug,
    ),
    component(
        "gateway",
        "Gateway",
        "edge",
        &["Nginx", "Envoy", "Kong", "AWS API Gateway", "Cloudflare"],
        Icon::Network,
    ),
    component(
        "worker",
        "Worker",
        "compute",
        &["Go", "Node.js", "Python", "Sidekiq", "Celery"],
        Icon::Cog,
    ),
    component(
        "function",
        "Function",
        "compute",
        &[
            "AWS Lambda",
            "Cloudflare Workers",
            "Vercel Functions",
            "Cloud Functions",
        ],
        Icon::Zap,
    ),
    component(
        "scheduler",
        "Scheduler",
        "workflow",
        &["Cron", "Temporal", "Airflow", "EventBridge Scheduler"],
        Icon::CalendarClock,
    ),
    component(
        "database",
        "Database",
        "storage",
        &[
            "Postgres",
            "MySQL",
            "SQLite",
            "MongoDB",
            "DynamoDB",
            "Cassandra",
        ],
        Icon::Database,
    ),
    component(
        "cache",
        "Cache",
        "storage",
        &["Redis", "Memcached", "Valkey"],
        Icon::MemoryStick,
    ),
    component(
        "queue",
        "Queue",
        "messaging",
        &["RabbitMQ", "SQS", "NATS", "BullMQ"],
        Icon::ListOrdered,
    ),
    component(
        "stream",
        "Event stream",
        "messaging",
        &["Kafka", "Kinesis", "Redpanda", "Pulsar"],
        Icon::Activity,
    ),
    component(
        "storage",
        "Object storage",
        "storage",
        &["S3", "R2", "GCS", "Azure Blob"],
        Icon::HardDrive,
    ),
    component(
        "search",
        "Search index",
        "storage",
        &[
            "Elasticsearch",
            "OpenSearch",
            "Meilisearch",
            "Typesense",
            "Algolia",
        ],
        Icon::Search,
    ),
    component(
        "warehouse",
        "Warehouse",
        "storage",
        &["BigQuery", "Snowflake", "Redshift", "ClickHouse", "DuckDB"],
        Icon::Warehouse,
    ),
    component(
        "auth",
        "Identity",
        "security",
        &["Auth0", "Clerk", "Cognito", "Keycloak", "Supabase Auth"],
        Icon::KeyRound,
    ),
    component(
        "cdn",
        "CDN",
        "edge",
        &["Cloudflare", "CloudFront", "Fastly", "Akamai"],
        Icon::Globe2,
    ),
    component(
        "cluster",
        "Cluster",
        "platform",
        &["Kubernetes", "ECS", "Nomad", "Docker Compose"],
        Icon::Cpu,
    ),
    component(
        "monitoring",
        "Monitoring",
        "observability",
        &[
            "Datadog",
            "Grafana",
            "Prometheus",
            "Sentry",
            "OpenTelemetry",
        ],
        Icon::Gauge,
    ),
    component(
        "external",
        "External API",
        "external",
        &["GitHub", "Slack", "Twilio", "Google APIs"],
        Icon::Cloud,
    ),
    component(
        "llm",
        "AI model",
        "ai",
        &["Claude", "GPT", "Gemini", "Llama", "Mistral"],
        Icon::AiBrain01Icon,
    ),
    component(
        "payment",
        "Payments",
        "external",
        &["Stripe", "Adyen", "PayPal", "Paddle"],
        Icon::CreditCard,
    ),
    component(
        "email",
        "Email",
        "external",
        &["SES", "SendGrid", "Postmark", "Resend", "Mailgun"],
        Icon::Inbox,
    ),
    component("user", "User", "people", &[], Icon::User),
    component("team", "Team", "people", &[], Icon::AddTeam),
    component(
        "agent",
        "Agent",
        "ai",
        &["Claude Code", "Codex", "Cursor", "LangGraph", "CrewAI"],
        Icon::RoboticIcon,
    ),
    component(
        "mcp",
        "MCP server",
        "ai",
        &[
            "Polterware MCP",
            "GitHub MCP",
            "Filesystem MCP",
            "Postgres MCP",
        ],
        Icon::McpServerIcon,
    ),
    component(
        "vector",
        "Vector index",
        "ai",
        &["pgvector", "Pinecone", "Weaviate", "Qdrant", "Chroma"],
        Icon::AiSearchIcon,
    ),
    component(
        "secrets",
        "Secrets",
        "security",
        &[
            "Vault",
            "AWS Secrets Manager",
            "Doppler",
            "1Password",
            "Infisical",
        ],
        Icon::SquareLock02Icon,
    ),
    component(
        "firewall",
        "Firewall",
        "security",
        &["Cloudflare WAF", "AWS WAF", "Fastly Next-Gen WAF"],
        Icon::FirewallIcon,
    ),
    component(
        "kms",
        "Key management",
        "security",
        &[
            "AWS KMS",
            "Google Cloud KMS",
            "Azure Key Vault",
            "Vault Transit",
        ],
        Icon::Key01Icon,
    ),
    component(
        "logs",
        "Logs",
        "observability",
        &[
            "Loki",
            "Elasticsearch",
            "CloudWatch Logs",
            "Datadog Logs",
            "Better Stack",
        ],
        Icon::ScrollText,
    ),
    component(
        "tracing",
        "Tracing",
        "observability",
        &[
            "OpenTelemetry",
            "Jaeger",
            "Tempo",
            "Honeycomb",
            "Datadog APM",
        ],
        Icon::Route,
    ),
    component(
        "alerting",
        "Alerting",
        "observability",
        &["PagerDuty", "Opsgenie", "Alertmanager", "incident.io"],
        Icon::Bell,
    ),
    component(
        "pipeline",
        "Pipeline",
        "workflow",
        &["Airflow", "Dagster", "dbt", "Fivetran", "Temporal"],
        Icon::Workflow,
    ),
    component(
        "ci",
        "CI/CD",
        "workflow",
        &[
            "GitHub Actions",
            "GitLab CI",
            "CircleCI",
            "Buildkite",
            "Jenkins",
        ],
        Icon::InfinityIcon,
    ),
    component(
        "webhook",
        "Webhook",
        "workflow",
        &["Stripe webhooks", "GitHub webhooks", "Svix", "Hookdeck"],
        Icon::Webhook,
    ),
    component(
        "config",
        "Config",
        "document",
        &["YAML", "JSON", "TOML", ".env", "Terraform"],
        Icon::SlidersHorizontalIcon,
    ),
    component(
        "spec",
        "Spec",
        "document",
        &["OpenAPI", "Markdown", "AsyncAPI", "Protobuf", "JSON Schema"],
        Icon::FileText,
    ),
    component(
        "dataset",
        "Dataset",
        "document",
        &["CSV", "Parquet", "JSON Lines", "Delta Lake", "Iceberg"],
        Icon::GridTableIcon,
    ),
];

pub fn component_of(kind: &str) -> Option<&'static Component> {
    COMPONENTS.iter().find(|item| item.kind == kind)
}

/// The family a component type belongs to.
pub fn family_for(kind: &str) -> Option<&'static Family> {
    component_of(kind).and_then(|item| family_of(item.family))
}

/// Domain and component nodes stand for Structure nodes and need a reference.
pub fn structure_kind(kind: &str) -> bool {
    kind == "domain" || kind == "component"
}

/// The silhouette a semantic node is drawn in: its family's, or Compute for Structure nodes.
pub fn silhouette(kind: &str) -> Option<&'static Family> {
    family_for(kind).or_else(|| structure_kind(kind).then(|| family_of("compute")).flatten())
}

/// The sign a semantic card carries: its component type's icon, or its Structure level's.
pub fn node_icon(kind: &str) -> Option<Icon> {
    component_of(kind).map(|item| item.icon).or(match kind {
        "domain" => Some(Icon::Boxes),
        "component" => Some(Icon::Blockchain01Icon),
        _ => None,
    })
}

/// Notes and text: annotations.
pub fn annotation(kind: &str) -> bool {
    kind == "note" || kind == "text"
}

/// Drawing shapes, notes, and text carry no meaning (`canvas.isFreeform`).
pub fn freeform(kind: &str) -> bool {
    annotation(kind) || shape_of(kind).is_some()
}

/// Capitalized, with dashes and underscores as spaces (`kindLabel`).
pub fn kind_label(kind: &str) -> String {
    let label = shape_of(kind)
        .map(|shape| shape.label.to_string())
        .or_else(|| component_of(kind).map(|item| item.label.to_string()))
        .unwrap_or_else(|| kind.replace(['-', '_'], " "));
    let mut chars = label.chars();
    match chars.next() {
        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
        None => String::new(),
    }
}

/// A library entry's name: a component type is offered as its family.
pub fn palette_label(kind: &str) -> String {
    family_for(kind)
        .map(|family| family.label.to_string())
        .unwrap_or_else(|| kind_label(kind))
}

/// The text a library search matches: a family matches each of its types and technologies,
/// so "redis" finds Storage.
pub fn palette_search_text(kind: &str) -> String {
    let family = component_of(kind).map(|item| item.family);
    let items: Vec<String> = COMPONENTS
        .iter()
        .filter(|item| Some(item.family) == family)
        .map(|item| {
            format!(
                "{} {} {}",
                item.kind,
                item.label,
                item.technologies.join(" ")
            )
        })
        .collect();
    format!("{kind} {} {}", palette_label(kind), items.join(" ")).to_lowercase()
}

/// The kinds a canvas allows (`nodePalette`): the drawing library when unrestricted, then
/// what the type lists.
pub fn node_palette(definition: &CanvasTypeDefinition) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    let library = definition
        .unrestricted
        .then(|| SHAPES.iter().map(|shape| shape.kind.to_string()))
        .into_iter()
        .flatten();
    for kind in library.chain(definition.node_kinds.iter().cloned()) {
        if !out.contains(&kind) {
            out.push(kind);
        }
    }
    out
}

/// The connection kinds a canvas allows (`edgePalette`).
pub fn edge_palette(definition: &CanvasTypeDefinition) -> Vec<String> {
    if !definition.unrestricted {
        return definition.edge_kinds.clone();
    }
    let mut out = vec!["relates".to_string()];
    for kind in &definition.edge_kinds {
        if !out.contains(kind) {
            out.push(kind.clone());
        }
    }
    out
}

/// One group of the shape library.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PaletteGroup {
    pub id: &'static str,
    pub title: &'static str,
    pub kinds: Vec<String>,
}

/// The library's groups (`paletteGroups`): one generic component per family, then the
/// drawing library and annotations; kinds a newer core adds fall into Other.
pub fn palette_groups(palette: &[String]) -> Vec<PaletteGroup> {
    let allowed = |kind: &str| palette.iter().any(|item| item == kind);
    let known = |kind: &str| freeform(kind) || component_of(kind).is_some() || structure_kind(kind);
    let library = |group: ShapeGroup| {
        SHAPES
            .iter()
            .filter(|shape| shape.group == group && allowed(shape.kind))
            .map(|shape| shape.kind.to_string())
            .collect::<Vec<_>>()
    };
    // A family's default type stands for it, or its first type the canvas allows.
    let components = FAMILIES
        .iter()
        .filter_map(|family| {
            let kinds: Vec<&str> = COMPONENTS
                .iter()
                .filter(|item| item.family == family.id && allowed(item.kind))
                .map(|item| item.kind)
                .collect();
            let first = *kinds.first()?;
            Some(if kinds.contains(&family.kind) {
                family.kind.to_string()
            } else {
                first.to_string()
            })
        })
        .collect();
    [
        PaletteGroup {
            id: "components",
            title: "Components",
            kinds: components,
        },
        PaletteGroup {
            id: "general",
            title: "General",
            kinds: library(ShapeGroup::General),
        },
        PaletteGroup {
            id: "flowchart",
            title: "Flowchart",
            kinds: library(ShapeGroup::Flowchart),
        },
        PaletteGroup {
            id: "annotations",
            title: "Annotations",
            kinds: palette
                .iter()
                .filter(|kind| annotation(kind))
                .cloned()
                .collect(),
        },
        PaletteGroup {
            id: "other",
            title: "Other",
            kinds: palette
                .iter()
                .filter(|kind| !known(kind))
                .cloned()
                .collect(),
        },
    ]
    .into_iter()
    .filter(|group| !group.kinds.is_empty())
    .collect()
}

/// How a node is drawn (`NodeVariant`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Variant {
    Card,
    Text,
    Sticky,
    Component,
}

/// The generalist canvas's node variant (`canvasDefaults.variant`).
pub fn variant(kind: &str) -> Variant {
    if kind == "note" {
        Variant::Sticky
    } else if component_of(kind).is_some() {
        Variant::Component
    } else if kind == "text" {
        Variant::Text
    } else {
        Variant::Card
    }
}

/// Whether a connection of `kind` has an arrow and a dashed line (`edgeStyle`).
pub fn edge_style(kind: &str) -> (bool, bool) {
    (kind != "relates", matches!(kind, "depends_on" | "contains"))
}

/// The size (and, for a frame, the stacking) a new node of `kind` starts with
/// (`initialGeometry`, then the drawing's own geometry), over the 180×100 default.
pub fn initial_geometry(kind: &str) -> (f32, f32, Option<i32>) {
    let (mut width, mut height, mut z) = (180.0, 100.0, None);
    if kind == "frame" {
        (width, height, z) = (440.0, 300.0, Some(-1));
    } else if kind == "note" {
        (width, height) = (200.0, 160.0);
    } else if let Some(family) = silhouette(kind) {
        (width, height) = family.geometry;
    }
    if let Some((w, h)) = shape_of(kind).and_then(|shape| shape.geometry) {
        (width, height) = (w, h);
    }
    (width, height, z)
}

/// The properties a new node of `kind` starts with (`canvasDefaults.defaults`).
pub fn default_props(kind: &str) -> serde_json::Map<String, serde_json::Value> {
    let value = match kind {
        "note" => serde_json::json!({ "color": "yellow" }),
        "frame" => serde_json::json!({
            "style": { "fill": "none", "dashed": true, "textAlign": "left", "bold": true }
        }),
        _ => serde_json::json!({}),
    };
    value.as_object().cloned().unwrap_or_default()
}

/// Frames sit behind connections and other shapes.
pub fn backdrop(kind: &str) -> bool {
    kind == "frame"
}

#[cfg(test)]
mod tests {
    use super::*;

    fn generalist() -> CanvasTypeDefinition {
        CanvasTypeDefinition {
            r#type: "canvas".into(),
            label: "Canvas".into(),
            node_kinds: ["note", "text", "domain", "component"]
                .into_iter()
                .map(String::from)
                .chain(COMPONENTS.iter().map(|item| item.kind.to_string()))
                .collect(),
            edge_kinds: vec!["calls".into(), "depends_on".into()],
            rules: Vec::new(),
            unrestricted: true,
        }
    }

    #[test]
    fn the_library_offers_one_component_per_family() {
        let groups = palette_groups(&node_palette(&generalist()));
        let ids: Vec<_> = groups.iter().map(|group| group.id).collect();
        assert_eq!(ids, ["components", "general", "flowchart", "annotations"]);
        assert_eq!(groups[0].kinds.len(), FAMILIES.len());
        assert_eq!(groups[0].kinds[0], "frontend");
        assert_eq!(groups[3].kinds, ["note", "text"]);
    }

    #[test]
    fn labels_and_search_read_as_the_electron_desktop_does() {
        assert_eq!(kind_label("manual-input"), "Manual input");
        assert_eq!(kind_label("depends_on"), "Depends on");
        assert_eq!(palette_label("database"), "Storage");
        assert!(palette_search_text("database").contains("redis"));
        assert_eq!(
            edge_palette(&generalist()),
            ["relates", "calls", "depends_on"]
        );
    }

    #[test]
    fn new_nodes_take_their_family_or_drawing_size() {
        assert_eq!(initial_geometry("database"), (200.0, 96.0, None));
        assert_eq!(initial_geometry("frame"), (440.0, 300.0, Some(-1)));
        assert_eq!(initial_geometry("diamond"), (160.0, 120.0, None));
        assert_eq!(initial_geometry("component"), (220.0, 64.0, None));
    }
}
