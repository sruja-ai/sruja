package language

import (
	"github.com/alecthomas/participle/v2/lexer"
)

// ============================================================================
// Deployment View
// ============================================================================

// DeploymentNode represents a deployment node (e.g., "Production", "AWS", "Server").
//
// Example DSL:
//
//	deployment "Production" {
//	  node "AWS" {
//	    node "US-East-1" {
//	       containerInstance WebApp
//	    }
//	  }
//	}
type DeploymentNode struct {
	Pos         lexer.Position
	Type        string               `parser:"@( 'deploymentNode' | 'deployment' | 'node' )"`
	ID          string               `parser:"@Ident"`
	Label       string               `parser:"@String"`
	Description *string              `parser:"( @String )?"`
	Items       []DeploymentNodeItem `parser:"( '{' @@* '}' )?"`

	// Post-processed
	Children           []*DeploymentNode
	ContainerInstances []*ContainerInstance
	Infrastructure     []*InfrastructureNode
}

func (d *DeploymentNode) Location() SourceLocation {
	return SourceLocation{File: d.Pos.Filename, Line: d.Pos.Line, Column: d.Pos.Column, Offset: d.Pos.Offset}
}

type DeploymentNodeItem struct {
	Node              *DeploymentNode     `parser:"@@"`
	ContainerInstance *ContainerInstance  `parser:"| @@"`
	Infrastructure    *InfrastructureNode `parser:"| @@"`
}

// ContainerInstance represents an instance of a container in a deployment node.
type ContainerInstance struct {
	Pos         lexer.Position
	ContainerID string  `parser:"'containerInstance' @Ident"`
	Label       string  `parser:"@String?"`
	InstanceID  *string `parser:"( 'instanceId' @Number )?"`
}

func (c *ContainerInstance) Location() SourceLocation {
	return SourceLocation{File: c.Pos.Filename, Line: c.Pos.Line, Column: c.Pos.Column, Offset: c.Pos.Offset}
}

// InfrastructureNode represents an infrastructure node (e.g., Load Balancer, DNS).
type InfrastructureNode struct {
	Pos         lexer.Position
	ID          string  `parser:"'infrastructure' @Ident"`
	Label       string  `parser:"@String"`
	Description *string `parser:"( @String )?"`
}

func (i *InfrastructureNode) Location() SourceLocation {
	return SourceLocation{File: i.Pos.Filename, Line: i.Pos.Line, Column: i.Pos.Column, Offset: i.Pos.Offset}
}
