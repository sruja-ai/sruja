# Storybook App Usage Guide

## Quick Start

### Installation
```bash
# Clone or navigate to the storybook app directory
cd storybook-app

# Install dependencies
npm install

# Start the development server
npm run dev
```

### Development Commands
```bash
npm run dev      # Start Storybook on http://localhost:6006
npm run build    # Build static Storybook for deployment
npm run preview  # Build and serve locally
```

## Usage Instructions

### 1. Browsing Components
- Navigate to `http://localhost:6006` after starting the development server
- Use the sidebar to browse components by category
- Click on any component to view its stories

### 2. Testing Components
- Use the **Controls** panel to modify component props in real-time
- Switch between different **Stories** to see component variations
- View the **Actions** tab to see event handlers firing

### 3. Viewing Documentation
- Click the **Docs** tab to see comprehensive component documentation
- View prop types, default values, and usage examples
- Copy code examples directly from the documentation

### 4. Searching Components
- Use the search bar in the top navigation
- Filter by component name or description
- Results update in real-time as you type

## Creating New Stories

### Basic Story Structure
```typescript
import type { Meta, StoryObj } from '@storybook/react'
import { Button } from '@sruja/ui'

const meta: Meta<typeof Button> = {
  title: 'Components/Button',
  component: Button,
  tags: ['autodocs'],
}

export default meta
type Story = StoryObj<typeof meta>

export const Primary: Story = {
  args: {
    variant: 'primary',
    children: 'Button Text',
  },
}
```

### Adding Controls
```typescript
export default {
  title: 'Components/Button',
  component: Button,
  argTypes: {
    variant: {
      control: { type: 'select' },
      options: ['primary', 'secondary', 'outline', 'ghost'],
    },
    size: {
      control: { type: 'radio' },
      options: ['sm', 'md', 'lg'],
    },
    disabled: {
      control: { type: 'boolean' },
    },
  },
}
```

### Writing Documentation
```typescript
export default {
  title: 'Components/Button',
  component: Button,
  parameters: {
    docs: {
      description: {
        component: 'Button component with multiple variants for different use cases.',
      },
    },
  },
}
```

## Best Practices

### 1. Story Naming
- Use descriptive story names (Primary, Secondary, Loading, Error)
- Group related stories together
- Keep story names concise but meaningful

### 2. Component Coverage
- Create stories for all component variants
- Include edge cases and error states
- Show components with different content lengths

### 3. Props Documentation
- Document all component props
- Include type information and default values
- Provide usage examples in descriptions

### 4. Interactive Testing
- Test component interactions
- Verify accessibility features
- Check responsive behavior

## Troubleshooting

### Common Issues

**Storybook won't start:**
- Check if port 6006 is available
- Verify all dependencies are installed
- Check for TypeScript errors

**Components not loading:**
- Ensure @sruja/ui is properly linked
- Check import paths in stories
- Verify component exports

**Controls not working:**
- Check argTypes configuration
- Ensure component props are properly typed
- Verify control types match prop types

### Getting Help
- Check the Storybook documentation at storybook.js.org
- Review the component library documentation
- Check for existing stories as examples

## Component Adoption Roadmap

### Phase 1: Foundation (Week 1-2)
**Studio Integration**
- Set up Storybook deployment pipeline
- Configure component library linking in Studio
- Create initial component showcase pages
- Train development team on Storybook workflow

**Documentation Migration**
- Migrate existing component documentation
- Create Storybook stories for core components
- Establish documentation standards and templates
- Set up automated documentation generation

### Phase 2: Expansion (Week 3-4)
**Component Coverage**
- Create stories for all UI components
- Add interactive examples and usage patterns
- Implement design token documentation
- Set up visual regression testing

**Cross-Platform Integration**
- Deploy Storybook for Docs platform
- Configure shared component library access
- Implement consistent theming across platforms
- Establish component versioning strategy

### Phase 3: Optimization (Week 5-6)
**Performance & Quality**
- Optimize build and deployment processes
- Implement automated testing workflows
- Set up component analytics and usage tracking
- Create maintenance and update procedures

**Team Adoption**
- Conduct training sessions for all teams
- Establish component contribution guidelines
- Create feedback and improvement processes
- Document best practices and conventions

## Accessibility and Visual Testing Plan

### Accessibility Testing
**Automated Testing**
- Integrate axe-core for automated accessibility audits
- Set up Storybook accessibility addon
- Configure CI/CD pipeline accessibility checks
- Test keyboard navigation and screen reader compatibility

**Manual Testing Checklist**
- Color contrast validation (WCAG 2.1 AA standards)
- Focus management and tab order verification
- Screen reader compatibility testing
- Keyboard-only navigation testing
- High contrast mode compatibility

**Component-Specific Tests**
- Form controls: labels, error messages, required field indicators
- Interactive elements: buttons, links, custom controls
- Dynamic content: modals, tooltips, notifications
- Media components: images, videos, audio content

### Visual Testing
**Visual Regression Testing**
- Implement Chromatic or Percy for visual regression testing
- Set up baseline screenshots for all component states
- Configure automated visual diff detection
- Establish visual testing approval workflows

**Cross-Browser Testing**
- Test components in Chrome, Firefox, Safari, Edge
- Verify mobile responsiveness across devices
- Validate component rendering at different viewport sizes
- Test dark mode and theme variations

**Design System Compliance**
- Verify adherence to design tokens and spacing
- Check typography consistency across components
- Validate color usage against brand guidelines
- Ensure icon and imagery consistency

### Testing Workflow
1. **Development Phase**: Run accessibility and visual tests locally
2. **Review Phase**: Automated tests run on pull requests
3. **Deployment Phase**: Full test suite runs before production
4. **Monitoring Phase**: Continuous monitoring post-deployment

## Migration Checklist

### Pre-Migration Assessment
- [ ] Audit existing component usage across Studio and Docs
- [ ] Document current component APIs and props
- [ ] Identify breaking changes and compatibility issues
- [ ] Create migration timeline and resource allocation
- [ ] Set up parallel development environment

### Migration Execution
- [ ] Migrate core components (Button, Input, Card, etc.)
- [ ] Update import statements and dependencies
- [ ] Refactor component usage patterns
- [ ] Update styling and theme configurations
- [ ] Migrate custom component extensions

### Testing and Validation
- [ ] Run comprehensive test suites on migrated components
- [ ] Validate visual consistency across platforms
- [ ] Test interactive functionality and user flows
- [ ] Verify accessibility compliance
- [ ] Performance testing and optimization

### Deployment and Rollback
- [ ] Staged deployment with feature flags
- [ ] Monitor application performance post-deployment
- [ ] Prepare rollback procedures for critical issues
- [ ] Document migration lessons learned
- [ ] Update development documentation and guides

### Post-Migration Tasks
- [ ] Archive old component library references
- [ ] Update team documentation and training materials
- [ ] Establish component governance processes
- [ ] Plan future component enhancements
- [ ] Schedule regular component library reviews