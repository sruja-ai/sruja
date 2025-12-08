package language

// PostProcess populates convenience fields from library items.
func (l *Library) PostProcess() {
	for _, item := range l.Items {
		if item.Description != nil {
			l.Description = item.Description
		}
		if item.Policy != nil {
			item.Policy.PostProcess()
			l.Policies = append(l.Policies, item.Policy)
		}
		if item.Requirement != nil {
			l.Requirements = append(l.Requirements, item.Requirement)
		}
		if item.Metadata != nil {
			l.Metadata = append(l.Metadata, item.Metadata.Entries...)
		}
	}
}
