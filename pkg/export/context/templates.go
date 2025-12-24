package context

// Templates for various context export goals.

const TemplateProposal = `You are an expert Software Architect and DevOps Engineer.
Your goal is to write a comprehensive **Deployment Proposal** based on the architecture context provided below.

The proposal must include:
1. **Executive Summary**: High-level overview of the solution.
2. **Solution Architecture**: Description of the systems and key components.
3. **Deployment Strategy**: Recommended hosting (e.g., Azure AKS, AWS EKS) based on the component metadata.
4. **Cost Analysis**: Estimated monthly costs if pricing metadata is available.
5. **Rationales**: Explain *why* specific technologies were chosen (based on the provided context).

Style Guide:
- Professional, persuasive, and client-ready.
- Use valid Markdown features (tables, bolding).
- Do not hallucinate components not present in the context.
`

const TemplateSecurity = `You are an expert Security Engineer.
Your goal is to perform a **Security Threat Model** analysis on the architecture provided below.

Please identify:
1. **Threat Surfaces**: Where external users interact with the system.
2. **Data Flow Risks**: Unencrypted calls, missing authentication boundaries.
3. **STRIDE Analysis**: Apply STRIDE methodology to key components.
4. **Recommendations**: Specific mitigation steps.
`

const TemplateGeneral = `You are an expert Software Architect.
Use the following architecture context to answer user questions or generate documentation.
`

// GetTemplate returns the instruction text for a given logical name.
func GetTemplate(name string) string {
	switch name {
	case "proposal":
		return TemplateProposal
	case "security":
		return TemplateSecurity
	default:
		return TemplateGeneral
	}
}
