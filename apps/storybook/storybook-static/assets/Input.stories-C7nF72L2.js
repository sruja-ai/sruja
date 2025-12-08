import{j as e}from"./iframe-CLHqt8sP.js";import{I as r}from"./Input-DMI3LCoc.js";import"./preload-helper-PPVm8Dsz.js";const m={title:"Components/Input",component:r,tags:["autodocs"],parameters:{layout:"centered",docs:{description:{component:"Enterprise-grade input with accessible label, helper and error messaging, and consistent focus styling aligned to the design system."}}},argTypes:{label:{control:{type:"text"},description:"Label text displayed above the input"},placeholder:{control:{type:"text"},description:"Placeholder text shown when input is empty"},helperText:{control:{type:"text"},description:"Helper text displayed below the input"},error:{control:{type:"text"},description:"Error message displayed below the input (overrides helperText)"},disabled:{control:{type:"boolean"},description:"Disables the input"},type:{control:{type:"select"},options:["text","email","password","number","tel","url"],description:"Input type"}}},a={args:{label:"Email",placeholder:"you@example.com",helperText:"We will never share your email."}},t={args:{label:"Project Name",placeholder:"Enter project name",helperText:"Choose a unique name for your project"},parameters:{docs:{description:{story:"Basic input with label and helper text."}}}},l={args:{label:"Email Address",placeholder:"you@example.com",error:"Please enter a valid email address",defaultValue:"invalid-email"},parameters:{docs:{description:{story:"Input in error state with validation message."}}}},s={render:()=>e.jsxs("div",{className:"space-y-4 max-w-md",children:[e.jsx(r,{type:"text",label:"Text Input",placeholder:"Enter text",helperText:"Standard text input"}),e.jsx(r,{type:"email",label:"Email Input",placeholder:"user@example.com",helperText:"Email address with validation"}),e.jsx(r,{type:"password",label:"Password",placeholder:"Enter password",helperText:"At least 8 characters"}),e.jsx(r,{type:"number",label:"Number Input",placeholder:"0",helperText:"Numeric value only"}),e.jsx(r,{type:"url",label:"Website URL",placeholder:"https://example.com",helperText:"Full URL including protocol"})]}),parameters:{docs:{description:{story:"Different input types for various data formats."}}}},o={render:()=>e.jsxs("div",{className:"space-y-4 max-w-md",children:[e.jsx(r,{label:"Default State",placeholder:"Type here...",helperText:"Normal input state"}),e.jsx(r,{label:"With Value",defaultValue:"Architecture as Code",helperText:"Input with pre-filled value"}),e.jsx(r,{label:"Error State",defaultValue:"invalid",error:"This field is required"}),e.jsx(r,{label:"Disabled State",defaultValue:"Cannot edit",disabled:!0,helperText:"This input is disabled"})]}),parameters:{docs:{description:{story:"All input states: default, with value, error, and disabled."}}}},d={render:()=>e.jsxs("form",{className:"space-y-4 max-w-md p-6 border border-[var(--color-border)] rounded-lg",children:[e.jsx("h3",{className:"text-lg font-semibold text-[var(--color-text-primary)] mb-4",children:"Create New Architecture"}),e.jsx(r,{label:"Architecture Name",placeholder:"e.g., E-commerce Platform",helperText:"Choose a descriptive name",required:!0}),e.jsx(r,{type:"email",label:"Contact Email",placeholder:"architect@example.com",helperText:"For notifications and updates"}),e.jsx(r,{label:"Description",placeholder:"Brief description of the architecture",helperText:"Optional description"}),e.jsxs("div",{className:"flex gap-3 pt-2",children:[e.jsx("button",{type:"submit",className:"px-5 py-2.5 bg-[var(--color-primary)] text-white rounded-md hover:opacity-90",children:"Create"}),e.jsx("button",{type:"button",className:"px-5 py-2.5 bg-transparent border border-[var(--color-border)] rounded-md",children:"Cancel"})]})]}),parameters:{docs:{description:{story:"Input components used in a complete form context."}}}},p={render:()=>e.jsxs("div",{className:"grid grid-cols-1 md:grid-cols-2 gap-6 max-w-2xl",children:[e.jsxs("div",{className:"bg-[var(--color-background)] border border-[var(--color-border)] rounded-lg shadow-sm p-5",children:[e.jsx("h4",{className:"text-base font-semibold mb-3",children:"States"}),e.jsxs("div",{className:"space-y-3",children:[e.jsx(r,{label:"Default",placeholder:"Type here",helperText:"Helper"}),e.jsx(r,{label:"Error",defaultValue:"invalid",error:"Required"}),e.jsx(r,{label:"Disabled",defaultValue:"Cannot edit",disabled:!0})]})]}),e.jsxs("div",{className:"bg-[var(--color-background)] border border-[var(--color-border)] rounded-lg shadow-sm p-5",children:[e.jsx("h4",{className:"text-base font-semibold mb-3",children:"Types"}),e.jsxs("div",{className:"space-y-3",children:[e.jsx(r,{type:"email",label:"Email",placeholder:"you@example.com"}),e.jsx(r,{type:"password",label:"Password",placeholder:"••••••••"}),e.jsx(r,{type:"number",label:"Number",placeholder:"0"})]})]})]})};a.parameters={...a.parameters,docs:{...a.parameters?.docs,source:{originalSource:`{
  args: {
    label: 'Email',
    placeholder: 'you@example.com',
    helperText: 'We will never share your email.'
  }
}`,...a.parameters?.docs?.source}}};t.parameters={...t.parameters,docs:{...t.parameters?.docs,source:{originalSource:`{
  args: {
    label: 'Project Name',
    placeholder: 'Enter project name',
    helperText: 'Choose a unique name for your project'
  },
  parameters: {
    docs: {
      description: {
        story: 'Basic input with label and helper text.'
      }
    }
  }
}`,...t.parameters?.docs?.source}}};l.parameters={...l.parameters,docs:{...l.parameters?.docs,source:{originalSource:`{
  args: {
    label: 'Email Address',
    placeholder: 'you@example.com',
    error: 'Please enter a valid email address',
    defaultValue: 'invalid-email'
  },
  parameters: {
    docs: {
      description: {
        story: 'Input in error state with validation message.'
      }
    }
  }
}`,...l.parameters?.docs?.source}}};s.parameters={...s.parameters,docs:{...s.parameters?.docs,source:{originalSource:`{
  render: () => <div className="space-y-4 max-w-md">
      <Input type="text" label="Text Input" placeholder="Enter text" helperText="Standard text input" />
      <Input type="email" label="Email Input" placeholder="user@example.com" helperText="Email address with validation" />
      <Input type="password" label="Password" placeholder="Enter password" helperText="At least 8 characters" />
      <Input type="number" label="Number Input" placeholder="0" helperText="Numeric value only" />
      <Input type="url" label="Website URL" placeholder="https://example.com" helperText="Full URL including protocol" />
    </div>,
  parameters: {
    docs: {
      description: {
        story: 'Different input types for various data formats.'
      }
    }
  }
}`,...s.parameters?.docs?.source}}};o.parameters={...o.parameters,docs:{...o.parameters?.docs,source:{originalSource:`{
  render: () => <div className="space-y-4 max-w-md">
      <Input label="Default State" placeholder="Type here..." helperText="Normal input state" />
      <Input label="With Value" defaultValue="Architecture as Code" helperText="Input with pre-filled value" />
      <Input label="Error State" defaultValue="invalid" error="This field is required" />
      <Input label="Disabled State" defaultValue="Cannot edit" disabled helperText="This input is disabled" />
    </div>,
  parameters: {
    docs: {
      description: {
        story: 'All input states: default, with value, error, and disabled.'
      }
    }
  }
}`,...o.parameters?.docs?.source}}};d.parameters={...d.parameters,docs:{...d.parameters?.docs,source:{originalSource:`{
  render: () => <form className="space-y-4 max-w-md p-6 border border-[var(--color-border)] rounded-lg">
      <h3 className="text-lg font-semibold text-[var(--color-text-primary)] mb-4">
        Create New Architecture
      </h3>
      <Input label="Architecture Name" placeholder="e.g., E-commerce Platform" helperText="Choose a descriptive name" required />
      <Input type="email" label="Contact Email" placeholder="architect@example.com" helperText="For notifications and updates" />
      <Input label="Description" placeholder="Brief description of the architecture" helperText="Optional description" />
      <div className="flex gap-3 pt-2">
        <button type="submit" className="px-5 py-2.5 bg-[var(--color-primary)] text-white rounded-md hover:opacity-90">
          Create
        </button>
        <button type="button" className="px-5 py-2.5 bg-transparent border border-[var(--color-border)] rounded-md">
          Cancel
        </button>
      </div>
    </form>,
  parameters: {
    docs: {
      description: {
        story: 'Input components used in a complete form context.'
      }
    }
  }
}`,...d.parameters?.docs?.source}}};p.parameters={...p.parameters,docs:{...p.parameters?.docs,source:{originalSource:`{
  render: () => <div className="grid grid-cols-1 md:grid-cols-2 gap-6 max-w-2xl">
      <div className="bg-[var(--color-background)] border border-[var(--color-border)] rounded-lg shadow-sm p-5">
        <h4 className="text-base font-semibold mb-3">States</h4>
        <div className="space-y-3">
          <Input label="Default" placeholder="Type here" helperText="Helper" />
          <Input label="Error" defaultValue="invalid" error="Required" />
          <Input label="Disabled" defaultValue="Cannot edit" disabled />
        </div>
      </div>
      <div className="bg-[var(--color-background)] border border-[var(--color-border)] rounded-lg shadow-sm p-5">
        <h4 className="text-base font-semibold mb-3">Types</h4>
        <div className="space-y-3">
          <Input type="email" label="Email" placeholder="you@example.com" />
          <Input type="password" label="Password" placeholder="••••••••" />
          <Input type="number" label="Number" placeholder="0" />
        </div>
      </div>
    </div>
}`,...p.parameters?.docs?.source}}};const u=["Playground","Basic","WithError","InputTypes","States","InForm","Showcase"];export{t as Basic,d as InForm,s as InputTypes,a as Playground,p as Showcase,o as States,l as WithError,u as __namedExportsOrder,m as default};
