import{j as e,a as r}from"./iframe-CLHqt8sP.js";import{c as h}from"./cn-VBtxfiI-.js";import{h as f}from"./dialog-BXVKZr_5.js";import{X as C}from"./x-B9rRl5ji.js";import{K as g}from"./transition-Bjg5DtQF.js";import{B as n}from"./Button-DrzFMabF.js";import{I as j}from"./Input-DMI3LCoc.js";import"./preload-helper-PPVm8Dsz.js";import"./bundle-mjs-COJ8Fh6m.js";import"./keyboard-DOqIamXD.js";import"./use-sync-refs-v6ctzwoA.js";import"./use-event-listener-CbHIemfg.js";import"./portal-CRTDVjVw.js";import"./focus-management-DE1YMz3g.js";import"./index-DRowAfMf.js";import"./index-dLArjxyT.js";import"./use-inert-others-D6T4iGKN.js";import"./use-tab-direction-H-bZp2mB.js";import"./hidden-CeoxBUgT.js";import"./close-provider-CmZQmh2t.js";import"./open-closed-B--VY9yg.js";import"./description-ezvdKXnU.js";import"./active-element-history-BLsi1N3R.js";import"./createLucideIcon-Byd03hVy.js";import"./variants-DKu3G6j2.js";const b={sm:"max-w-[400px] w-[90vw]",md:"max-w-[600px] w-[90vw]",lg:"max-w-[800px] w-[90vw]",xl:"max-w-[1024px] w-[90vw]",full:"max-w-screen w-screen h-screen rounded-none"};function l({isOpen:s,onClose:t,title:o,children:i,footer:a,size:v="md",showCloseButton:x=!0,className:y=""}){return e.jsx(g,{appear:!0,show:s,as:r.Fragment,children:e.jsxs(f,{as:"div",className:"relative z-50",onClose:t,children:[e.jsx(g.Child,{as:r.Fragment,enter:"ease-out duration-300",enterFrom:"opacity-0",enterTo:"opacity-100",leave:"ease-in duration-200",leaveFrom:"opacity-100",leaveTo:"opacity-0",children:e.jsx("div",{className:"fixed inset-0 bg-black/50","aria-hidden":"true"})}),e.jsx("div",{className:"fixed inset-0 overflow-y-auto",children:e.jsx("div",{className:"flex min-h-full items-center justify-center p-4",children:e.jsx(g.Child,{as:r.Fragment,enter:"ease-out duration-300",enterFrom:"opacity-0 scale-95",enterTo:"opacity-100 scale-100",leave:"ease-in duration-200",leaveFrom:"opacity-100 scale-100",leaveTo:"opacity-0 scale-95",children:e.jsxs(f.Panel,{className:h("transform overflow-hidden transition-all","bg-[var(--color-background)] rounded-lg shadow-xl",b[v],y),children:[(o||x)&&e.jsxs("div",{className:"px-6 py-4 border-b border-[var(--color-border)] flex justify-between items-center",children:[o&&e.jsx(f.Title,{as:"h3",className:"m-0 text-xl font-semibold text-[var(--color-text-primary)]",children:o}),x&&e.jsx("button",{onClick:t,className:"p-1 text-[var(--color-text-tertiary)] rounded hover:bg-[var(--color-surface)] hover:text-[var(--color-text-primary)] transition-all focus:outline-none focus-visible:ring-2 focus-visible:ring-offset-2 focus-visible:ring-[var(--color-primary)]","aria-label":"Close dialog",children:e.jsx(C,{size:20})})]}),e.jsx("div",{className:"p-6 text-[var(--color-text-primary)]",children:i}),a&&e.jsx("div",{className:"px-6 py-4 border-t border-[var(--color-border)] flex justify-end gap-3",children:a})]})})})})]})})}l.__docgenInfo={description:"",methods:[],displayName:"Dialog",props:{isOpen:{required:!0,tsType:{name:"boolean"},description:"Whether the dialog is open"},onClose:{required:!0,tsType:{name:"signature",type:"function",raw:"() => void",signature:{arguments:[],return:{name:"void"}}},description:"Callback when dialog should close"},title:{required:!1,tsType:{name:"string"},description:"Dialog title"},children:{required:!0,tsType:{name:"ReactNode"},description:"Dialog content"},footer:{required:!1,tsType:{name:"ReactNode"},description:"Custom footer content"},size:{required:!1,tsType:{name:"union",raw:"'sm' | 'md' | 'lg' | 'xl' | 'full'",elements:[{name:"literal",value:"'sm'"},{name:"literal",value:"'md'"},{name:"literal",value:"'lg'"},{name:"literal",value:"'xl'"},{name:"literal",value:"'full'"}]},description:"Dialog size",defaultValue:{value:"'md'",computed:!1}},showCloseButton:{required:!1,tsType:{name:"boolean"},description:"Show close button",defaultValue:{value:"true",computed:!1}},className:{required:!1,tsType:{name:"string"},description:"Additional CSS classes",defaultValue:{value:"''",computed:!1}}}};const H={title:"Components/Dialog",component:l,tags:["autodocs"],parameters:{layout:"centered",docs:{description:{component:"A modal dialog component for displaying important information, forms, or confirmations. Used throughout Sruja Studio for user interactions like adding elements, exporting diagrams, and confirming actions."}}},argTypes:{isOpen:{control:{type:"boolean"},description:"Controls dialog visibility"},title:{control:{type:"text"},description:"Dialog title"},onClose:{action:"closed",description:"Callback fired when dialog should close"}}},d={render:()=>{const[s,t]=r.useState(!1);return e.jsxs("div",{style:{padding:16},children:[e.jsx(n,{onClick:()=>t(!0),children:"Open Dialog"}),e.jsx(l,{isOpen:s,onClose:()=>t(!1),title:"Dialog Title",footer:e.jsxs("div",{style:{display:"flex",gap:8,justifyContent:"flex-end"},children:[e.jsx(n,{variant:"ghost",onClick:()=>t(!1),children:"Cancel"}),e.jsx(n,{onClick:()=>t(!1),children:"Confirm"})]}),children:"Dialog content goes here."})]})}},c={render:()=>{const[s,t]=r.useState(!1),[o,i]=r.useState("");return e.jsxs("div",{style:{padding:16},children:[e.jsx(n,{onClick:()=>t(!0),children:"Add Container"}),e.jsx(l,{isOpen:s,onClose:()=>{t(!1),i("")},title:"Add Container",footer:e.jsxs("div",{style:{display:"flex",gap:8,justifyContent:"flex-end"},children:[e.jsx(n,{variant:"ghost",onClick:()=>t(!1),children:"Cancel"}),e.jsx(n,{onClick:()=>{alert(`Added container: ${o}`),t(!1),i("")},disabled:!o.trim(),children:"Add"})]}),children:e.jsx("div",{style:{padding:"8px 0"},children:e.jsx(j,{label:"Container Name",placeholder:"e.g., API Service",value:o,onChange:a=>i(a.target.value),helperText:"Enter a descriptive name for the container"})})})]})},parameters:{docs:{description:{story:"Dialog for adding new architecture elements with input validation."}}}},p={render:()=>{const[s,t]=r.useState(!1);return e.jsxs("div",{style:{padding:16},children:[e.jsx(n,{variant:"danger",onClick:()=>t(!0),children:"Delete System"}),e.jsx(l,{isOpen:s,onClose:()=>t(!1),title:"Delete System",footer:e.jsxs("div",{style:{display:"flex",gap:8,justifyContent:"flex-end"},children:[e.jsx(n,{variant:"ghost",onClick:()=>t(!1),children:"Cancel"}),e.jsx(n,{variant:"danger",onClick:()=>{alert("System deleted"),t(!1)},children:"Delete"})]}),children:e.jsx("p",{style:{margin:0,color:"var(--color-text-secondary)"},children:'Are you sure you want to delete "Web Application"? This action cannot be undone and will remove all containers and relations.'})})]})},parameters:{docs:{description:{story:"Confirmation dialog for destructive actions like deleting architecture elements."}}}},m={render:()=>{const[s,t]=r.useState(!1),[o,i]=r.useState("png");return e.jsxs("div",{style:{padding:16},children:[e.jsx(n,{onClick:()=>t(!0),children:"Export Diagram"}),e.jsx(l,{isOpen:s,onClose:()=>t(!1),title:"Export Diagram",footer:e.jsxs("div",{style:{display:"flex",gap:8,justifyContent:"flex-end"},children:[e.jsx(n,{variant:"ghost",onClick:()=>t(!1),children:"Cancel"}),e.jsx(n,{onClick:()=>{alert(`Exporting as ${o.toUpperCase()}`),t(!1)},children:"Export"})]}),children:e.jsxs("div",{style:{display:"flex",flexDirection:"column",gap:16},children:[e.jsxs("div",{children:[e.jsx("label",{style:{display:"block",marginBottom:8,fontSize:14,fontWeight:500},children:"Format"}),e.jsx("div",{style:{display:"flex",gap:8},children:["png","svg","json"].map(a=>e.jsx("button",{onClick:()=>i(a),style:{flex:1,padding:"8px 16px",borderRadius:6,border:`2px solid ${o===a?"#3b82f6":"#e2e8f0"}`,backgroundColor:o===a?"#eff6ff":"#fff",color:o===a?"#1d4ed8":"#475569",cursor:"pointer",fontSize:14,fontWeight:o===a?600:500,textTransform:"uppercase"},children:a},a))})]}),e.jsx("div",{children:e.jsxs("label",{style:{display:"flex",alignItems:"center",gap:8,cursor:"pointer"},children:[e.jsx("input",{type:"checkbox",defaultChecked:!0}),e.jsx("span",{style:{fontSize:14},children:"Include metadata"})]})})]})})]})},parameters:{docs:{description:{story:"Dialog for exporting architecture diagrams with format selection and options."}}}},u={render:()=>{const[s,t]=r.useState(!1);return e.jsxs("div",{children:[e.jsx(n,{onClick:()=>t(!0),children:"Open Dialog"}),e.jsx(l,{isOpen:s,onClose:()=>t(!1),title:"Modern Dialog",footer:e.jsxs("div",{style:{display:"flex",gap:8,justifyContent:"flex-end"},children:[e.jsx(n,{variant:"ghost",onClick:()=>t(!1),children:"Cancel"}),e.jsx(n,{onClick:()=>t(!1),children:"Confirm"})]}),children:e.jsx("div",{className:"text-sm text-[var(--color-text-secondary)]",children:"Dialog content aligned with design-system spacing, borders, and typography."})})]})}};d.parameters={...d.parameters,docs:{...d.parameters?.docs,source:{originalSource:`{
  render: () => {
    const [open, setOpen] = useState(false);
    return <div style={{
      padding: 16
    }}>
        <Button onClick={() => setOpen(true)}>Open Dialog</Button>
        <Dialog isOpen={open} onClose={() => setOpen(false)} title="Dialog Title" footer={<div style={{
        display: 'flex',
        gap: 8,
        justifyContent: 'flex-end'
      }}>
              <Button variant="ghost" onClick={() => setOpen(false)}>Cancel</Button>
              <Button onClick={() => setOpen(false)}>Confirm</Button>
            </div>}>
          Dialog content goes here.
        </Dialog>
      </div>;
  }
}`,...d.parameters?.docs?.source}}};c.parameters={...c.parameters,docs:{...c.parameters?.docs,source:{originalSource:`{
  render: () => {
    const [open, setOpen] = useState(false);
    const [name, setName] = useState('');
    return <div style={{
      padding: 16
    }}>
        <Button onClick={() => setOpen(true)}>Add Container</Button>
        <Dialog isOpen={open} onClose={() => {
        setOpen(false);
        setName('');
      }} title="Add Container" footer={<div style={{
        display: 'flex',
        gap: 8,
        justifyContent: 'flex-end'
      }}>
              <Button variant="ghost" onClick={() => setOpen(false)}>Cancel</Button>
              <Button onClick={() => {
          alert(\`Added container: \${name}\`);
          setOpen(false);
          setName('');
        }} disabled={!name.trim()}>
                Add
              </Button>
            </div>}>
          <div style={{
          padding: '8px 0'
        }}>
            <Input label="Container Name" placeholder="e.g., API Service" value={name} onChange={e => setName(e.target.value)} helperText="Enter a descriptive name for the container" />
          </div>
        </Dialog>
      </div>;
  },
  parameters: {
    docs: {
      description: {
        story: 'Dialog for adding new architecture elements with input validation.'
      }
    }
  }
}`,...c.parameters?.docs?.source}}};p.parameters={...p.parameters,docs:{...p.parameters?.docs,source:{originalSource:`{
  render: () => {
    const [open, setOpen] = useState(false);
    return <div style={{
      padding: 16
    }}>
        <Button variant="danger" onClick={() => setOpen(true)}>Delete System</Button>
        <Dialog isOpen={open} onClose={() => setOpen(false)} title="Delete System" footer={<div style={{
        display: 'flex',
        gap: 8,
        justifyContent: 'flex-end'
      }}>
              <Button variant="ghost" onClick={() => setOpen(false)}>Cancel</Button>
              <Button variant="danger" onClick={() => {
          alert('System deleted');
          setOpen(false);
        }}>
                Delete
              </Button>
            </div>}>
          <p style={{
          margin: 0,
          color: 'var(--color-text-secondary)'
        }}>
            Are you sure you want to delete "Web Application"? This action cannot be undone and will remove all containers and relations.
          </p>
        </Dialog>
      </div>;
  },
  parameters: {
    docs: {
      description: {
        story: 'Confirmation dialog for destructive actions like deleting architecture elements.'
      }
    }
  }
}`,...p.parameters?.docs?.source}}};m.parameters={...m.parameters,docs:{...m.parameters?.docs,source:{originalSource:`{
  render: () => {
    const [open, setOpen] = useState(false);
    const [format, setFormat] = useState<'png' | 'svg' | 'json'>('png');
    return <div style={{
      padding: 16
    }}>
        <Button onClick={() => setOpen(true)}>Export Diagram</Button>
        <Dialog isOpen={open} onClose={() => setOpen(false)} title="Export Diagram" footer={<div style={{
        display: 'flex',
        gap: 8,
        justifyContent: 'flex-end'
      }}>
              <Button variant="ghost" onClick={() => setOpen(false)}>Cancel</Button>
              <Button onClick={() => {
          alert(\`Exporting as \${format.toUpperCase()}\`);
          setOpen(false);
        }}>
                Export
              </Button>
            </div>}>
          <div style={{
          display: 'flex',
          flexDirection: 'column',
          gap: 16
        }}>
            <div>
              <label style={{
              display: 'block',
              marginBottom: 8,
              fontSize: 14,
              fontWeight: 500
            }}>
                Format
              </label>
              <div style={{
              display: 'flex',
              gap: 8
            }}>
                {(['png', 'svg', 'json'] as const).map(fmt => <button key={fmt} onClick={() => setFormat(fmt)} style={{
                flex: 1,
                padding: '8px 16px',
                borderRadius: 6,
                border: \`2px solid \${format === fmt ? '#3b82f6' : '#e2e8f0'}\`,
                backgroundColor: format === fmt ? '#eff6ff' : '#fff',
                color: format === fmt ? '#1d4ed8' : '#475569',
                cursor: 'pointer',
                fontSize: 14,
                fontWeight: format === fmt ? 600 : 500,
                textTransform: 'uppercase'
              }}>
                    {fmt}
                  </button>)}
              </div>
            </div>
            <div>
              <label style={{
              display: 'flex',
              alignItems: 'center',
              gap: 8,
              cursor: 'pointer'
            }}>
                <input type="checkbox" defaultChecked />
                <span style={{
                fontSize: 14
              }}>Include metadata</span>
              </label>
            </div>
          </div>
        </Dialog>
      </div>;
  },
  parameters: {
    docs: {
      description: {
        story: 'Dialog for exporting architecture diagrams with format selection and options.'
      }
    }
  }
}`,...m.parameters?.docs?.source}}};u.parameters={...u.parameters,docs:{...u.parameters?.docs,source:{originalSource:`{
  render: () => {
    const [open, setOpen] = useState(false);
    return <div>
        <Button onClick={() => setOpen(true)}>Open Dialog</Button>
        <Dialog isOpen={open} onClose={() => setOpen(false)} title="Modern Dialog" footer={<div style={{
        display: 'flex',
        gap: 8,
        justifyContent: 'flex-end'
      }}><Button variant="ghost" onClick={() => setOpen(false)}>Cancel</Button><Button onClick={() => setOpen(false)}>Confirm</Button></div>}>
          <div className="text-sm text-[var(--color-text-secondary)]">Dialog content aligned with design-system spacing, borders, and typography.</div>
        </Dialog>
      </div>;
  }
}`,...u.parameters?.docs?.source}}};const J=["Playground","AddElement","Confirmation","ExportOptions","Showcase"];export{c as AddElement,p as Confirmation,m as ExportOptions,d as Playground,u as Showcase,J as __namedExportsOrder,H as default};
