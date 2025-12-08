import{j as e}from"./iframe-CLHqt8sP.js";import{B as r}from"./Button-DrzFMabF.js";import"./preload-helper-PPVm8Dsz.js";import"./variants-DKu3G6j2.js";import"./bundle-mjs-COJ8Fh6m.js";const x={title:"Components/Button",component:r,tags:["autodocs"],parameters:{layout:"centered",docs:{description:{component:"Enterprise-grade button with clear hierarchy, robust focus states, and accessible variants. Designed to align with modern product UI patterns."}}},argTypes:{variant:{control:{type:"select"},options:["primary","secondary","outline","ghost","danger"],description:"Visual style variant of the button",table:{type:{summary:"primary | secondary | outline | ghost | danger"},defaultValue:{summary:"primary"}}},size:{control:{type:"radio"},options:["sm","md","lg"],description:"Size of the button",table:{type:{summary:"sm | md | lg"},defaultValue:{summary:"md"}}},isLoading:{control:{type:"boolean"},description:"Shows loading spinner and disables interaction",table:{type:{summary:"boolean"},defaultValue:{summary:"false"}}},disabled:{control:{type:"boolean"},description:"Disables the button",table:{type:{summary:"boolean"},defaultValue:{summary:"false"}}},children:{control:{type:"text"},description:"Button label or content"}}},a={args:{children:"Button",variant:"primary",size:"md",isLoading:!1}},t={render:()=>e.jsxs("div",{className:"flex flex-wrap gap-4 items-center",children:[e.jsx(r,{variant:"primary",children:"Primary Action"}),e.jsx(r,{variant:"secondary",children:"Secondary Action"}),e.jsx(r,{variant:"outline",children:"Outline Button"}),e.jsx(r,{variant:"ghost",children:"Ghost Button"}),e.jsx(r,{variant:"danger",children:"Delete"})]}),parameters:{docs:{description:{story:"All available button variants for different use cases and visual hierarchy."}}}},s={render:()=>e.jsxs("div",{className:"flex flex-wrap gap-4 items-center",children:[e.jsx(r,{size:"sm",children:"Small Button"}),e.jsx(r,{size:"md",children:"Medium Button"}),e.jsx(r,{size:"lg",children:"Large Button"})]}),parameters:{docs:{description:{story:"Three size options to match different UI contexts and importance levels."}}}},o={render:()=>e.jsxs("div",{className:"flex flex-wrap gap-4 items-center",children:[e.jsx(r,{children:"Default"}),e.jsx(r,{isLoading:!0,children:"Loading"}),e.jsx(r,{disabled:!0,children:"Disabled"})]}),parameters:{docs:{description:{story:"Button states including default, loading, and disabled states."}}}},n={render:()=>e.jsxs("div",{className:"space-y-4",children:[e.jsxs("div",{className:"flex gap-3",children:[e.jsx(r,{variant:"primary",size:"lg",children:"Create Architecture"}),e.jsx(r,{variant:"primary",size:"lg",isLoading:!0,children:"Exporting..."})]}),e.jsx("p",{className:"text-sm text-[var(--color-text-secondary)]",children:"Use primary buttons for the main action on a page or in a section."})]}),parameters:{docs:{description:{story:"Primary buttons are used for the most important actions like creating, saving, or submitting."}}}},i={render:()=>e.jsxs("div",{className:"space-y-4",children:[e.jsxs("div",{className:"flex gap-3",children:[e.jsx(r,{variant:"secondary",children:"Cancel"}),e.jsx(r,{variant:"outline",children:"Learn More"}),e.jsx(r,{variant:"ghost",children:"View Details"})]}),e.jsx("p",{className:"text-sm text-[var(--color-text-secondary)]",children:"Secondary buttons are used for less prominent actions or alternatives to primary actions."})]}),parameters:{docs:{description:{story:"Secondary button variants for supporting actions and navigation."}}}},d={render:()=>e.jsxs("div",{className:"space-y-4",children:[e.jsxs("div",{className:"flex gap-3",children:[e.jsx(r,{variant:"danger",children:"Delete Architecture"}),e.jsx(r,{variant:"danger",disabled:!0,children:"Cannot Delete"})]}),e.jsx("p",{className:"text-sm text-[var(--color-text-secondary)]",children:"Use danger variant for destructive actions that require user confirmation."})]}),parameters:{docs:{description:{story:"Danger buttons for destructive actions like deletion or removal."}}}},c={render:()=>e.jsxs("div",{className:"max-w-md space-y-4 p-6 border border-[var(--color-border)] rounded-lg",children:[e.jsx("h3",{className:"text-lg font-semibold text-[var(--color-text-primary)]",children:"Create New Project"}),e.jsxs("div",{className:"space-y-3",children:[e.jsx("input",{type:"text",placeholder:"Project name",className:"w-full px-3.5 py-2.5 rounded-md border border-[var(--color-border)] bg-[var(--color-background)]"}),e.jsx("textarea",{placeholder:"Description",rows:3,className:"w-full px-3.5 py-2.5 rounded-md border border-[var(--color-border)] bg-[var(--color-background)]"})]}),e.jsxs("div",{className:"flex gap-3 pt-2",children:[e.jsx(r,{variant:"primary",children:"Create Project"}),e.jsx(r,{variant:"ghost",children:"Cancel"})]})]}),parameters:{docs:{description:{story:"Button usage in forms with primary action and cancel option."}}}},l={render:()=>e.jsxs("div",{className:"space-y-6",children:[e.jsxs("div",{className:"grid grid-cols-1 gap-6 md:grid-cols-2",children:[e.jsxs("div",{className:"bg-[var(--color-background)] border border-[var(--color-border)] rounded-lg shadow-sm p-5",children:[e.jsx("h4",{className:"text-base font-semibold mb-3",children:"Primary & Secondary"}),e.jsxs("div",{className:"flex flex-wrap gap-3 items-center",children:[e.jsx(r,{variant:"primary",children:"Primary"}),e.jsx(r,{variant:"primary",isLoading:!0,children:"Loading"}),e.jsx(r,{variant:"secondary",children:"Secondary"}),e.jsx(r,{variant:"outline",children:"Outline"}),e.jsx(r,{variant:"ghost",children:"Ghost"})]})]}),e.jsxs("div",{className:"bg-[var(--color-background)] border border-[var(--color-border)] rounded-lg shadow-sm p-5",children:[e.jsx("h4",{className:"text-base font-semibold mb-3",children:"Sizes"}),e.jsxs("div",{className:"flex flex-wrap gap-3 items-center",children:[e.jsx(r,{size:"sm",children:"Small"}),e.jsx(r,{size:"md",children:"Medium"}),e.jsx(r,{size:"lg",children:"Large"})]})]}),e.jsxs("div",{className:"bg-[var(--color-background)] border border-[var(--color-border)] rounded-lg shadow-sm p-5",children:[e.jsx("h4",{className:"text-base font-semibold mb-3",children:"States"}),e.jsxs("div",{className:"flex flex-wrap gap-3 items-center",children:[e.jsx(r,{children:"Default"}),e.jsx(r,{disabled:!0,children:"Disabled"})]})]}),e.jsxs("div",{className:"bg-[var(--color-background)] border border-[var(--color-border)] rounded-lg shadow-sm p-5",children:[e.jsx("h4",{className:"text-base font-semibold mb-3",children:"Destructive"}),e.jsxs("div",{className:"flex flex-wrap gap-3 items-center",children:[e.jsx(r,{variant:"danger",children:"Delete"}),e.jsx(r,{variant:"danger",disabled:!0,children:"Delete (Disabled)"})]})]})]}),e.jsx("p",{className:"text-sm text-[var(--color-text-secondary)]",children:"This showcase demonstrates consistent spacing, typography, and elevations aligned with a modern product system."})]})};a.parameters={...a.parameters,docs:{...a.parameters?.docs,source:{originalSource:`{
  args: {
    children: 'Button',
    variant: 'primary',
    size: 'md',
    isLoading: false
  }
}`,...a.parameters?.docs?.source}}};t.parameters={...t.parameters,docs:{...t.parameters?.docs,source:{originalSource:`{
  render: () => <div className="flex flex-wrap gap-4 items-center">
      <Button variant="primary">Primary Action</Button>
      <Button variant="secondary">Secondary Action</Button>
      <Button variant="outline">Outline Button</Button>
      <Button variant="ghost">Ghost Button</Button>
      <Button variant="danger">Delete</Button>
    </div>,
  parameters: {
    docs: {
      description: {
        story: 'All available button variants for different use cases and visual hierarchy.'
      }
    }
  }
}`,...t.parameters?.docs?.source}}};s.parameters={...s.parameters,docs:{...s.parameters?.docs,source:{originalSource:`{
  render: () => <div className="flex flex-wrap gap-4 items-center">
      <Button size="sm">Small Button</Button>
      <Button size="md">Medium Button</Button>
      <Button size="lg">Large Button</Button>
    </div>,
  parameters: {
    docs: {
      description: {
        story: 'Three size options to match different UI contexts and importance levels.'
      }
    }
  }
}`,...s.parameters?.docs?.source}}};o.parameters={...o.parameters,docs:{...o.parameters?.docs,source:{originalSource:`{
  render: () => <div className="flex flex-wrap gap-4 items-center">
      <Button>Default</Button>
      <Button isLoading>Loading</Button>
      <Button disabled>Disabled</Button>
    </div>,
  parameters: {
    docs: {
      description: {
        story: 'Button states including default, loading, and disabled states.'
      }
    }
  }
}`,...o.parameters?.docs?.source}}};n.parameters={...n.parameters,docs:{...n.parameters?.docs,source:{originalSource:`{
  render: () => <div className="space-y-4">
      <div className="flex gap-3">
        <Button variant="primary" size="lg">Create Architecture</Button>
        <Button variant="primary" size="lg" isLoading>Exporting...</Button>
      </div>
      <p className="text-sm text-[var(--color-text-secondary)]">
        Use primary buttons for the main action on a page or in a section.
      </p>
    </div>,
  parameters: {
    docs: {
      description: {
        story: 'Primary buttons are used for the most important actions like creating, saving, or submitting.'
      }
    }
  }
}`,...n.parameters?.docs?.source}}};i.parameters={...i.parameters,docs:{...i.parameters?.docs,source:{originalSource:`{
  render: () => <div className="space-y-4">
      <div className="flex gap-3">
        <Button variant="secondary">Cancel</Button>
        <Button variant="outline">Learn More</Button>
        <Button variant="ghost">View Details</Button>
      </div>
      <p className="text-sm text-[var(--color-text-secondary)]">
        Secondary buttons are used for less prominent actions or alternatives to primary actions.
      </p>
    </div>,
  parameters: {
    docs: {
      description: {
        story: 'Secondary button variants for supporting actions and navigation.'
      }
    }
  }
}`,...i.parameters?.docs?.source}}};d.parameters={...d.parameters,docs:{...d.parameters?.docs,source:{originalSource:`{
  render: () => <div className="space-y-4">
      <div className="flex gap-3">
        <Button variant="danger">Delete Architecture</Button>
        <Button variant="danger" disabled>Cannot Delete</Button>
      </div>
      <p className="text-sm text-[var(--color-text-secondary)]">
        Use danger variant for destructive actions that require user confirmation.
      </p>
    </div>,
  parameters: {
    docs: {
      description: {
        story: 'Danger buttons for destructive actions like deletion or removal.'
      }
    }
  }
}`,...d.parameters?.docs?.source}}};c.parameters={...c.parameters,docs:{...c.parameters?.docs,source:{originalSource:`{
  render: () => <div className="max-w-md space-y-4 p-6 border border-[var(--color-border)] rounded-lg">
      <h3 className="text-lg font-semibold text-[var(--color-text-primary)]">Create New Project</h3>
      <div className="space-y-3">
        <input type="text" placeholder="Project name" className="w-full px-3.5 py-2.5 rounded-md border border-[var(--color-border)] bg-[var(--color-background)]" />
        <textarea placeholder="Description" rows={3} className="w-full px-3.5 py-2.5 rounded-md border border-[var(--color-border)] bg-[var(--color-background)]" />
      </div>
      <div className="flex gap-3 pt-2">
        <Button variant="primary">Create Project</Button>
        <Button variant="ghost">Cancel</Button>
      </div>
    </div>,
  parameters: {
    docs: {
      description: {
        story: 'Button usage in forms with primary action and cancel option.'
      }
    }
  }
}`,...c.parameters?.docs?.source}}};l.parameters={...l.parameters,docs:{...l.parameters?.docs,source:{originalSource:`{
  render: () => <div className="space-y-6">
      <div className="grid grid-cols-1 gap-6 md:grid-cols-2">
        <div className="bg-[var(--color-background)] border border-[var(--color-border)] rounded-lg shadow-sm p-5">
          <h4 className="text-base font-semibold mb-3">Primary & Secondary</h4>
          <div className="flex flex-wrap gap-3 items-center">
            <Button variant="primary">Primary</Button>
            <Button variant="primary" isLoading>Loading</Button>
            <Button variant="secondary">Secondary</Button>
            <Button variant="outline">Outline</Button>
            <Button variant="ghost">Ghost</Button>
          </div>
        </div>
        <div className="bg-[var(--color-background)] border border-[var(--color-border)] rounded-lg shadow-sm p-5">
          <h4 className="text-base font-semibold mb-3">Sizes</h4>
          <div className="flex flex-wrap gap-3 items-center">
            <Button size="sm">Small</Button>
            <Button size="md">Medium</Button>
            <Button size="lg">Large</Button>
          </div>
        </div>
        <div className="bg-[var(--color-background)] border border-[var(--color-border)] rounded-lg shadow-sm p-5">
          <h4 className="text-base font-semibold mb-3">States</h4>
          <div className="flex flex-wrap gap-3 items-center">
            <Button>Default</Button>
            <Button disabled>Disabled</Button>
          </div>
        </div>
        <div className="bg-[var(--color-background)] border border-[var(--color-border)] rounded-lg shadow-sm p-5">
          <h4 className="text-base font-semibold mb-3">Destructive</h4>
          <div className="flex flex-wrap gap-3 items-center">
            <Button variant="danger">Delete</Button>
            <Button variant="danger" disabled>Delete (Disabled)</Button>
          </div>
        </div>
      </div>
      <p className="text-sm text-[var(--color-text-secondary)]">
        This showcase demonstrates consistent spacing, typography, and elevations aligned with a modern product system.
      </p>
    </div>
}`,...l.parameters?.docs?.source}}};const h=["Playground","Variants","Sizes","States","PrimaryActions","SecondaryActions","DestructiveActions","InForms","Showcase"];export{d as DestructiveActions,c as InForms,a as Playground,n as PrimaryActions,i as SecondaryActions,l as Showcase,s as Sizes,o as States,t as Variants,h as __namedExportsOrder,x as default};
