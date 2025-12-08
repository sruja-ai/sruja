import{j as e}from"./iframe-CLHqt8sP.js";import{H as c,D as f}from"./download-DlmzcEjP.js";import{F as p}from"./Footer-B5cUFjW8.js";import{T as h}from"./ThemeToggle-DOgjCiVE.js";import{B as o}from"./Button-DrzFMabF.js";import{C as r}from"./Card-BHe2Chf6.js";import{B as t}from"./Badge-DGs5HKmX.js";import"./preload-helper-PPVm8Dsz.js";import"./Logo-CKDpWWdM.js";import"./cn-VBtxfiI-.js";import"./bundle-mjs-COJ8Fh6m.js";import"./createLucideIcon-Byd03hVy.js";import"./variants-DKu3G6j2.js";import"./transition-Bjg5DtQF.js";import"./use-sync-refs-v6ctzwoA.js";import"./open-closed-B--VY9yg.js";function a({header:n,sidebar:l,children:m,footer:d}){return e.jsxs("div",{className:"min-h-screen bg-[var(--color-surface)] text-[var(--color-text-primary)]",children:[n&&e.jsx("div",{className:"border-b border-[var(--color-border)] bg-[var(--color-background)]",children:n}),e.jsxs("div",{className:"flex",children:[l&&e.jsx("aside",{className:"w-64 border-r border-[var(--color-border)] bg-[var(--color-background)]",children:l}),e.jsx("main",{className:"flex-1 p-6",children:m})]}),d&&e.jsx("div",{className:"border-t border-[var(--color-border)] bg-[var(--color-background)]",children:d})]})}a.__docgenInfo={description:"",methods:[],displayName:"AppShell",props:{header:{required:!1,tsType:{name:"any"},description:""},sidebar:{required:!1,tsType:{name:"any"},description:""},children:{required:!0,tsType:{name:"any"},description:""},footer:{required:!1,tsType:{name:"any"},description:""}}};const R={title:"Components/AppShell",component:a,tags:["autodocs"],parameters:{docs:{description:{component:"Complete application shell layout used in Sruja Studio. Provides header, sidebar, main content area, and footer. This is the foundation for the full Studio application interface."}},layout:"fullscreen"}},i={render:()=>e.jsx("div",{style:{height:"600px",border:"1px solid var(--color-border)",borderRadius:8,overflow:"hidden"},children:e.jsx(a,{header:e.jsx(c,{title:"Sruja Studio",subtitle:"Architecture Visualization Tool",version:"0.1.0",leftContent:e.jsx("div",{}),rightContent:e.jsx(h,{iconOnly:!0,size:"sm"})}),sidebar:e.jsxs("div",{style:{padding:16},children:[e.jsx("h3",{style:{fontSize:12,fontWeight:600,textTransform:"uppercase",color:"var(--color-text-secondary)",marginBottom:12},children:"Explorer"}),e.jsxs("div",{style:{display:"flex",flexDirection:"column",gap:4},children:[e.jsx(o,{variant:"ghost",size:"sm",style:{justifyContent:"flex-start"},children:"Web Application"}),e.jsx(o,{variant:"ghost",size:"sm",style:{justifyContent:"flex-start"},children:"API Service"}),e.jsx(o,{variant:"ghost",size:"sm",style:{justifyContent:"flex-start"},children:"Database"})]})]}),footer:e.jsx(p,{leftContent:e.jsxs("span",{children:["© ",new Date().getFullYear()," Sruja"]}),centerContent:e.jsx("span",{children:"Architecture as Code"}),rightContent:e.jsx("a",{href:"https://sruja.ai",target:"_blank",rel:"noopener noreferrer",children:"sruja.ai"})}),children:e.jsxs("div",{style:{padding:24},children:[e.jsx("h2",{style:{marginTop:0,marginBottom:16},children:"Architecture Overview"}),e.jsxs("div",{style:{display:"grid",gridTemplateColumns:"repeat(auto-fit, minmax(250px, 1fr))",gap:16},children:[e.jsx(r,{title:"Systems",footer:e.jsx(t,{color:"brand",children:"3"}),children:e.jsx("p",{style:{fontSize:14,color:"var(--color-text-secondary)",margin:0},children:"Total systems defined in architecture"})}),e.jsx(r,{title:"Containers",footer:e.jsx(t,{children:"12"}),children:e.jsx("p",{style:{fontSize:14,color:"var(--color-text-secondary)",margin:0},children:"Container components across all systems"})}),e.jsx(r,{title:"Relations",footer:e.jsx(t,{color:"info",children:"24"}),children:e.jsx("p",{style:{fontSize:14,color:"var(--color-text-secondary)",margin:0},children:"Connections between elements"})})]})]})})}),parameters:{docs:{description:{story:"Complete application shell with header, sidebar navigation, main content, and footer."}}}},s={render:()=>e.jsx("div",{style:{height:"700px",border:"1px solid var(--color-border)",borderRadius:8,overflow:"hidden"},children:e.jsx(a,{header:e.jsx(c,{title:"Sruja Studio",subtitle:"Architecture Visualization Tool",version:"0.1.0",leftContent:e.jsxs("div",{style:{display:"flex",alignItems:"center",gap:12},children:[e.jsx("label",{style:{fontSize:14,fontWeight:500,color:"var(--color-text-secondary)"},children:"Example:"}),e.jsxs("select",{style:{padding:"6px 12px",borderRadius:6,border:"1px solid var(--color-border)",fontSize:14,backgroundColor:"var(--color-background)"},children:[e.jsx("option",{children:"Simple Web App"}),e.jsx("option",{children:"E-commerce Platform"})]})]}),rightContent:e.jsxs("div",{style:{display:"flex",alignItems:"center",gap:8},children:[e.jsx(o,{variant:"ghost",size:"sm",children:"Preview"}),e.jsxs(o,{size:"sm",style:{display:"flex",alignItems:"center",gap:6},children:[e.jsx(f,{size:16}),"Export"]}),e.jsx(h,{iconOnly:!0,size:"sm"})]})}),sidebar:e.jsxs("div",{style:{padding:16,height:"100%",borderRight:"1px solid var(--color-border)"},children:[e.jsx("h3",{style:{fontSize:12,fontWeight:600,textTransform:"uppercase",color:"var(--color-text-secondary)",marginBottom:16},children:"Model Explorer"}),e.jsxs("div",{style:{display:"flex",flexDirection:"column",gap:8},children:[e.jsxs("div",{style:{padding:8,backgroundColor:"var(--color-surface)",borderRadius:6},children:[e.jsx("div",{style:{fontWeight:600,fontSize:14,marginBottom:4},children:"Web Application"}),e.jsx("div",{style:{fontSize:12,color:"var(--color-text-secondary)"},children:"System"})]}),e.jsxs("div",{style:{paddingLeft:16},children:[e.jsx("div",{style:{padding:6,fontSize:13},children:"API Service"}),e.jsx("div",{style:{padding:6,fontSize:13},children:"Database"})]})]})]}),footer:e.jsx(p,{leftContent:e.jsxs("span",{children:["© ",new Date().getFullYear()," Sruja"]}),centerContent:e.jsx("span",{children:"Architecture as Code"}),rightContent:e.jsx("a",{href:"https://sruja.ai",target:"_blank",rel:"noopener noreferrer",children:"sruja.ai"})}),children:e.jsxs("div",{style:{padding:24,height:"100%",display:"flex",flexDirection:"column",gap:16},children:[e.jsx("div",{style:{flex:1,border:"1px dashed var(--color-border)",borderRadius:8,display:"flex",alignItems:"center",justifyContent:"center",backgroundColor:"var(--color-surface)"},children:e.jsxs("div",{style:{textAlign:"center",color:"var(--color-text-secondary)"},children:[e.jsx("div",{style:{fontSize:18,fontWeight:600,marginBottom:8},children:"Architecture Diagram"}),e.jsx("div",{style:{fontSize:14},children:"Interactive viewer would appear here"})]})}),e.jsxs("div",{style:{display:"grid",gridTemplateColumns:"repeat(3, 1fr)",gap:12},children:[e.jsx(r,{title:"Elements",footer:e.jsx(t,{children:"15"}),children:e.jsx("div",{style:{fontSize:24,fontWeight:600},children:"15"})}),e.jsx(r,{title:"Relations",footer:e.jsx(t,{color:"info",children:"8"}),children:e.jsx("div",{style:{fontSize:24,fontWeight:600},children:"8"})}),e.jsx(r,{title:"Status",footer:e.jsx(t,{color:"success",children:"Valid"}),children:e.jsx("div",{style:{fontSize:14,color:"var(--color-text-secondary)"},children:"No errors"})})]})]})})}),parameters:{docs:{description:{story:"Complete Studio layout with model explorer sidebar and architecture viewer."}}}};i.parameters={...i.parameters,docs:{...i.parameters?.docs,source:{originalSource:`{
  render: () => <div style={{
    height: '600px',
    border: '1px solid var(--color-border)',
    borderRadius: 8,
    overflow: 'hidden'
  }}>
      <AppShell header={<Header title="Sruja Studio" subtitle="Architecture Visualization Tool" version="0.1.0" leftContent={<div />} rightContent={<ThemeToggle iconOnly size="sm" />} />} sidebar={<div style={{
      padding: 16
    }}>
            <h3 style={{
        fontSize: 12,
        fontWeight: 600,
        textTransform: 'uppercase',
        color: 'var(--color-text-secondary)',
        marginBottom: 12
      }}>
              Explorer
            </h3>
            <div style={{
        display: 'flex',
        flexDirection: 'column',
        gap: 4
      }}>
              <Button variant="ghost" size="sm" style={{
          justifyContent: 'flex-start'
        }}>Web Application</Button>
              <Button variant="ghost" size="sm" style={{
          justifyContent: 'flex-start'
        }}>API Service</Button>
              <Button variant="ghost" size="sm" style={{
          justifyContent: 'flex-start'
        }}>Database</Button>
            </div>
          </div>} footer={<Footer leftContent={<span>© {new Date().getFullYear()} Sruja</span>} centerContent={<span>Architecture as Code</span>} rightContent={<a href="https://sruja.ai" target="_blank" rel="noopener noreferrer">sruja.ai</a>} />}>
        <div style={{
        padding: 24
      }}>
          <h2 style={{
          marginTop: 0,
          marginBottom: 16
        }}>Architecture Overview</h2>
          <div style={{
          display: 'grid',
          gridTemplateColumns: 'repeat(auto-fit, minmax(250px, 1fr))',
          gap: 16
        }}>
            <Card title="Systems" footer={<Badge color="brand">3</Badge>}>
              <p style={{
              fontSize: 14,
              color: 'var(--color-text-secondary)',
              margin: 0
            }}>
                Total systems defined in architecture
              </p>
            </Card>
            <Card title="Containers" footer={<Badge>12</Badge>}>
              <p style={{
              fontSize: 14,
              color: 'var(--color-text-secondary)',
              margin: 0
            }}>
                Container components across all systems
              </p>
            </Card>
            <Card title="Relations" footer={<Badge color="info">24</Badge>}>
              <p style={{
              fontSize: 14,
              color: 'var(--color-text-secondary)',
              margin: 0
            }}>
                Connections between elements
              </p>
            </Card>
          </div>
        </div>
      </AppShell>
    </div>,
  parameters: {
    docs: {
      description: {
        story: 'Complete application shell with header, sidebar navigation, main content, and footer.'
      }
    }
  }
}`,...i.parameters?.docs?.source}}};s.parameters={...s.parameters,docs:{...s.parameters?.docs,source:{originalSource:`{
  render: () => <div style={{
    height: '700px',
    border: '1px solid var(--color-border)',
    borderRadius: 8,
    overflow: 'hidden'
  }}>
      <AppShell header={<Header title="Sruja Studio" subtitle="Architecture Visualization Tool" version="0.1.0" leftContent={<div style={{
      display: 'flex',
      alignItems: 'center',
      gap: 12
    }}>
                <label style={{
        fontSize: 14,
        fontWeight: 500,
        color: 'var(--color-text-secondary)'
      }}>
                  Example:
                </label>
                <select style={{
        padding: '6px 12px',
        borderRadius: 6,
        border: '1px solid var(--color-border)',
        fontSize: 14,
        backgroundColor: 'var(--color-background)'
      }}>
                  <option>Simple Web App</option>
                  <option>E-commerce Platform</option>
                </select>
              </div>} rightContent={<div style={{
      display: 'flex',
      alignItems: 'center',
      gap: 8
    }}>
                <Button variant="ghost" size="sm">Preview</Button>
                <Button size="sm" style={{
        display: 'flex',
        alignItems: 'center',
        gap: 6
      }}>
                  <Download size={16} />
                  Export
                </Button>
                <ThemeToggle iconOnly size="sm" />
              </div>} />} sidebar={<div style={{
      padding: 16,
      height: '100%',
      borderRight: '1px solid var(--color-border)'
    }}>
            <h3 style={{
        fontSize: 12,
        fontWeight: 600,
        textTransform: 'uppercase',
        color: 'var(--color-text-secondary)',
        marginBottom: 16
      }}>
              Model Explorer
            </h3>
            <div style={{
        display: 'flex',
        flexDirection: 'column',
        gap: 8
      }}>
              <div style={{
          padding: 8,
          backgroundColor: 'var(--color-surface)',
          borderRadius: 6
        }}>
                <div style={{
            fontWeight: 600,
            fontSize: 14,
            marginBottom: 4
          }}>Web Application</div>
                <div style={{
            fontSize: 12,
            color: 'var(--color-text-secondary)'
          }}>System</div>
              </div>
              <div style={{
          paddingLeft: 16
        }}>
                <div style={{
            padding: 6,
            fontSize: 13
          }}>API Service</div>
                <div style={{
            padding: 6,
            fontSize: 13
          }}>Database</div>
              </div>
            </div>
          </div>} footer={<Footer leftContent={<span>© {new Date().getFullYear()} Sruja</span>} centerContent={<span>Architecture as Code</span>} rightContent={<a href="https://sruja.ai" target="_blank" rel="noopener noreferrer">sruja.ai</a>} />}>
        <div style={{
        padding: 24,
        height: '100%',
        display: 'flex',
        flexDirection: 'column',
        gap: 16
      }}>
          <div style={{
          flex: 1,
          border: '1px dashed var(--color-border)',
          borderRadius: 8,
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'center',
          backgroundColor: 'var(--color-surface)'
        }}>
            <div style={{
            textAlign: 'center',
            color: 'var(--color-text-secondary)'
          }}>
              <div style={{
              fontSize: 18,
              fontWeight: 600,
              marginBottom: 8
            }}>Architecture Diagram</div>
              <div style={{
              fontSize: 14
            }}>Interactive viewer would appear here</div>
            </div>
          </div>
          <div style={{
          display: 'grid',
          gridTemplateColumns: 'repeat(3, 1fr)',
          gap: 12
        }}>
            <Card title="Elements" footer={<Badge>15</Badge>}>
              <div style={{
              fontSize: 24,
              fontWeight: 600
            }}>15</div>
            </Card>
            <Card title="Relations" footer={<Badge color="info">8</Badge>}>
              <div style={{
              fontSize: 24,
              fontWeight: 600
            }}>8</div>
            </Card>
            <Card title="Status" footer={<Badge color="success">Valid</Badge>}>
              <div style={{
              fontSize: 14,
              color: 'var(--color-text-secondary)'
            }}>No errors</div>
            </Card>
          </div>
        </div>
      </AppShell>
    </div>,
  parameters: {
    docs: {
      description: {
        story: 'Complete Studio layout with model explorer sidebar and architecture viewer.'
      }
    }
  }
}`,...s.parameters?.docs?.source}}};const k=["Playground","StudioLayout"];export{i as Playground,s as StudioLayout,k as __namedExportsOrder,R as default};
