//! Shared kind enums and source-binding types.

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ElementKind {
    Person,
    Role,
    System,
    Container,
    Component,
    Database,
    Queue,
    ExternalSystem,
    DataStore,
    Policy,
    Requirement,
    Adr,
    Flow,
    Scenario,
    Story,
    Custom(String),
}

impl std::fmt::Display for ElementKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ElementKind::Person => write!(f, "person"),
            ElementKind::Role => write!(f, "role"),
            ElementKind::System => write!(f, "system"),
            ElementKind::Container => write!(f, "container"),
            ElementKind::Component => write!(f, "component"),
            ElementKind::Database => write!(f, "database"),
            ElementKind::Queue => write!(f, "queue"),
            ElementKind::ExternalSystem => write!(f, "externalSystem"),
            ElementKind::DataStore => write!(f, "datastore"),
            ElementKind::Policy => write!(f, "policy"),
            ElementKind::Requirement => write!(f, "requirement"),
            ElementKind::Adr => write!(f, "adr"),
            ElementKind::Flow => write!(f, "flow"),
            ElementKind::Scenario => write!(f, "scenario"),
            ElementKind::Story => write!(f, "story"),
            ElementKind::Custom(k) => write!(f, "{k}"),
        }
    }
}

#[derive(Debug, Clone, thiserror::Error, PartialEq, Eq)]
#[error("Unknown NodeKind: '{0}'")]
pub struct ParseNodeKindError(pub String);

#[derive(Debug, Clone, thiserror::Error, PartialEq, Eq)]
#[error("Unknown EdgeKind: '{0}'")]
pub struct ParseEdgeKindError(pub String);

#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[serde(transparent)]
pub struct NodeKind(pub String);

impl NodeKind {
    pub fn new(s: impl Into<String>) -> Self {
        Self(s.into())
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn kind_str(&self) -> &str {
        &self.0
    }

    pub fn to_string_kind(&self) -> String {
        self.0.clone()
    }

    pub fn is_custom(&self) -> bool {
        !matches!(
            self.as_str(),
            "system"
                | "service"
                | "container"
                | "component"
                | "database"
                | "queue"
                | "external_api"
                | "frontend"
                | "module"
        )
    }

    pub const SYSTEM: &'static str = "system";
    pub const SERVICE: &'static str = "service";
    pub const CONTAINER: &'static str = "container";
    pub const COMPONENT: &'static str = "component";
    pub const DATABASE: &'static str = "database";
    pub const QUEUE: &'static str = "queue";
    pub const EXTERNAL_API: &'static str = "external_api";
    pub const FRONTEND: &'static str = "frontend";
    pub const MODULE: &'static str = "module";
}

impl std::fmt::Display for NodeKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl std::str::FromStr for NodeKind {
    type Err = ParseNodeKindError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(NodeKind(s.to_string()))
    }
}

impl PartialEq<&str> for NodeKind {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}

impl PartialEq<str> for NodeKind {
    fn eq(&self, other: &str) -> bool {
        self.as_str() == other
    }
}

impl PartialEq<String> for NodeKind {
    fn eq(&self, other: &String) -> bool {
        self.as_str() == other.as_str()
    }
}

impl From<&str> for NodeKind {
    fn from(s: &str) -> Self {
        NodeKind(s.to_string())
    }
}

impl From<String> for NodeKind {
    fn from(s: String) -> Self {
        NodeKind(s)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[serde(transparent)]
pub struct EdgeKind(pub String);

impl EdgeKind {
    pub fn new(s: impl Into<String>) -> Self {
        Self(s.into())
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn kind_str(&self) -> &str {
        &self.0
    }

    pub fn to_string_kind(&self) -> String {
        self.0.clone()
    }

    pub fn is_custom(&self) -> bool {
        !matches!(
            self.as_str(),
            "depends_on"
                | "calls"
                | "reads_from"
                | "writes_to"
                | "publishes_to"
                | "subscribes_to"
                | "owns"
                | "contains"
                | "uses"
        )
    }

    pub const DEPENDS_ON: &'static str = "depends_on";
    pub const CALLS: &'static str = "calls";
    pub const READS_FROM: &'static str = "reads_from";
    pub const WRITES_TO: &'static str = "writes_to";
    pub const PUBLISHES_TO: &'static str = "publishes_to";
    pub const SUBSCRIBES_TO: &'static str = "subscribes_to";
    pub const OWNS: &'static str = "owns";
    pub const CONTAINS: &'static str = "contains";
    pub const USES: &'static str = "uses";
}

impl std::fmt::Display for EdgeKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl std::str::FromStr for EdgeKind {
    type Err = ParseEdgeKindError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(EdgeKind(s.to_string()))
    }
}

impl PartialEq<&str> for EdgeKind {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}

impl PartialEq<str> for EdgeKind {
    fn eq(&self, other: &str) -> bool {
        self.as_str() == other
    }
}

impl PartialEq<String> for EdgeKind {
    fn eq(&self, other: &String) -> bool {
        self.as_str() == other.as_str()
    }
}

impl From<&str> for EdgeKind {
    fn from(s: &str) -> Self {
        EdgeKind(s.to_string())
    }
}

impl From<String> for EdgeKind {
    fn from(s: String) -> Self {
        EdgeKind(s)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Criticality {
    Low,
    Medium,
    High,
    Critical,
}

impl Criticality {
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Criticality::Low => "low",
            Criticality::Medium => "medium",
            Criticality::High => "high",
            Criticality::Critical => "critical",
        }
    }
}

impl std::fmt::Display for Criticality {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl std::str::FromStr for Criticality {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "low" => Ok(Criticality::Low),
            "medium" | "med" => Ok(Criticality::Medium),
            "high" => Ok(Criticality::High),
            "critical" => Ok(Criticality::Critical),
            _ => Err(format!("Unknown criticality: {s}")),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SourceKind {
    OpenApi,
    AsyncApi,
    Kubernetes,
    Dockerfile,
    Terraform,
    Docs,
    Readme,
    Proto,
    Config,
    GraphQL,
    Helm,
    Custom(String),
}

impl SourceKind {
    #[must_use]
    pub fn as_str(&self) -> &str {
        match self {
            SourceKind::OpenApi => "openapi",
            SourceKind::AsyncApi => "asyncapi",
            SourceKind::Kubernetes => "kubernetes",
            SourceKind::Dockerfile => "dockerfile",
            SourceKind::Terraform => "terraform",
            SourceKind::Docs => "docs",
            SourceKind::Readme => "readme",
            SourceKind::Proto => "proto",
            SourceKind::Config => "config",
            SourceKind::GraphQL => "graphql",
            SourceKind::Helm => "helm",
            SourceKind::Custom(s) => s,
        }
    }

    #[must_use]
    pub fn parse(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "openapi" => SourceKind::OpenApi,
            "asyncapi" => SourceKind::AsyncApi,
            "kubernetes" | "k8s" => SourceKind::Kubernetes,
            "dockerfile" | "docker" => SourceKind::Dockerfile,
            "terraform" | "tf" => SourceKind::Terraform,
            "docs" | "doc" => SourceKind::Docs,
            "readme" => SourceKind::Readme,
            "proto" | "protobuf" => SourceKind::Proto,
            "config" => SourceKind::Config,
            "graphql" | "gql" => SourceKind::GraphQL,
            "helm" => SourceKind::Helm,
            _ => SourceKind::Custom(s.to_string()),
        }
    }
}

impl std::str::FromStr for SourceKind {
    type Err = std::convert::Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(SourceKind::parse(s))
    }
}

impl std::fmt::Display for SourceKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct SourceBinding {
    pub kind: SourceKind,
    pub path: String,
    pub description: Option<String>,
}
