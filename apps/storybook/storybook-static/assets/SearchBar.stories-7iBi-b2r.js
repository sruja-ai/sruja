import{a as n,j as s}from"./iframe-CLHqt8sP.js";import{S as u}from"./SearchBar-DOqKo0pP.js";import{B as b}from"./Badge-DGs5HKmX.js";import"./preload-helper-PPVm8Dsz.js";import"./combobox-CkWSAl-G.js";import"./useFocusRing-4vPDgUpM.js";import"./index-DRowAfMf.js";import"./index-dLArjxyT.js";import"./use-resolve-button-type-DhPqIc8n.js";import"./keyboard-DOqIamXD.js";import"./use-sync-refs-v6ctzwoA.js";import"./use-by-comparator-Brosz9J5.js";import"./form-fields-Djd7uIxW.js";import"./hidden-CeoxBUgT.js";import"./floating-2JlZ9ldt.js";import"./element-movement-D0iZG3pA.js";import"./bugs-ByA1MTr8.js";import"./portal-CRTDVjVw.js";import"./focus-management-DE1YMz3g.js";import"./use-inert-others-D6T4iGKN.js";import"./use-event-listener-CbHIemfg.js";import"./open-closed-B--VY9yg.js";import"./use-tree-walker-D3zlMO2p.js";import"./active-element-history-BLsi1N3R.js";import"./description-ezvdKXnU.js";import"./frozen-CSHtb0Sm.js";import"./label-TZAP9ERS.js";const H={title:"Components/SearchBar",component:u,tags:["autodocs"],parameters:{docs:{description:{component:"Search bar component for finding architecture elements. Used in Sruja Studio for quickly locating systems, containers, and other elements in large architectures. Supports keyboard navigation and filtering."}}},argTypes:{query:{control:{type:"text"},description:"Current search query"},onQueryChange:{action:"query changed",description:"Callback fired when search query changes"},onSelect:{action:"selected",description:"Callback fired when a result is selected"}}},m=[{id:"customer",label:"Customer",subLabel:"Person"},{id:"admin",label:"Administrator",subLabel:"Person"},{id:"ecommerce",label:"E-commerce Platform",subLabel:"System"},{id:"webapp",label:"Web Application",subLabel:"E-commerce Platform > Container"},{id:"api",label:"API Gateway",subLabel:"E-commerce Platform > Container"},{id:"payment",label:"Payment Service",subLabel:"E-commerce Platform > Container"},{id:"userdb",label:"User Database",subLabel:"E-commerce Platform > Datastore"},{id:"productdb",label:"Product Database",subLabel:"E-commerce Platform > Datastore"}],l={render:()=>{const[r,a]=n.useState(""),[t,e]=n.useState(null),o=n.useMemo(()=>{const d=r.trim().toLowerCase();return d?m.filter(p=>p.label.toLowerCase().includes(d)||(p.subLabel||"").toLowerCase().includes(d)):[]},[r]);return s.jsxs("div",{style:{display:"flex",flexDirection:"column",gap:16,maxWidth:600},children:[s.jsx(u,{query:r,onQueryChange:a,results:o,onSelect:e}),t&&s.jsxs("div",{style:{padding:12,backgroundColor:"var(--color-surface)",borderRadius:6,fontSize:14},children:[s.jsx("strong",{children:"Selected:"})," ",t.label," ",s.jsx(b,{color:"info",style:{marginLeft:8,fontSize:11},children:t.subLabel})]})]})}},i={render:()=>{const[r,a]=n.useState(""),t=n.useMemo(()=>{const e=r.trim().toLowerCase();return e?m.filter(o=>o.label.toLowerCase().includes(e)||(o.subLabel||"").toLowerCase().includes(e)):[]},[r]);return s.jsx("div",{style:{maxWidth:500},children:s.jsx(u,{query:r,onQueryChange:a,results:t,onSelect:e=>alert(`Selected: ${e.label}`)})})},parameters:{docs:{description:{story:"Basic search bar for finding architecture elements by name or type."}}}},c={render:()=>{const[r,a]=n.useState("api"),t=n.useMemo(()=>{const e=r.trim().toLowerCase();return e?m.filter(o=>o.label.toLowerCase().includes(e)||(o.subLabel||"").toLowerCase().includes(e)):[]},[r]);return s.jsxs("div",{style:{maxWidth:500},children:[s.jsx(u,{query:r,onQueryChange:a,results:t,onSelect:e=>alert(`Selected: ${e.label}`)}),t.length>0&&s.jsxs("div",{style:{marginTop:8,fontSize:12,color:"var(--color-text-secondary)"},children:["Found ",t.length," ",t.length===1?"result":"results"]})]})},parameters:{docs:{description:{story:"Search bar with pre-filled query showing filtered results."}}}};l.parameters={...l.parameters,docs:{...l.parameters?.docs,source:{originalSource:`{
  render: () => {
    const [q, setQ] = useState('');
    const [sel, setSel] = useState<SearchItem | null>(null);
    const results = useMemo(() => {
      const s = q.trim().toLowerCase();
      if (!s) return [];
      return ARCHITECTURE_ELEMENTS.filter(i => i.label.toLowerCase().includes(s) || (i.subLabel || '').toLowerCase().includes(s));
    }, [q]);
    return <div style={{
      display: 'flex',
      flexDirection: 'column',
      gap: 16,
      maxWidth: 600
    }}>
        <SearchBar query={q} onQueryChange={setQ} results={results} onSelect={setSel} />
        {sel && <div style={{
        padding: 12,
        backgroundColor: 'var(--color-surface)',
        borderRadius: 6,
        fontSize: 14
      }}>
            <strong>Selected:</strong> {sel.label} <Badge color="info" style={{
          marginLeft: 8,
          fontSize: 11
        }}>{sel.subLabel}</Badge>
          </div>}
      </div>;
  }
}`,...l.parameters?.docs?.source}}};i.parameters={...i.parameters,docs:{...i.parameters?.docs,source:{originalSource:`{
  render: () => {
    const [q, setQ] = useState('');
    const results = useMemo(() => {
      const s = q.trim().toLowerCase();
      if (!s) return [];
      return ARCHITECTURE_ELEMENTS.filter(i => i.label.toLowerCase().includes(s) || (i.subLabel || '').toLowerCase().includes(s));
    }, [q]);
    return <div style={{
      maxWidth: 500
    }}>
        <SearchBar query={q} onQueryChange={setQ} results={results} onSelect={item => alert(\`Selected: \${item.label}\`)} />
      </div>;
  },
  parameters: {
    docs: {
      description: {
        story: 'Basic search bar for finding architecture elements by name or type.'
      }
    }
  }
}`,...i.parameters?.docs?.source}}};c.parameters={...c.parameters,docs:{...c.parameters?.docs,source:{originalSource:`{
  render: () => {
    const [q, setQ] = useState('api');
    const results = useMemo(() => {
      const s = q.trim().toLowerCase();
      if (!s) return [];
      return ARCHITECTURE_ELEMENTS.filter(i => i.label.toLowerCase().includes(s) || (i.subLabel || '').toLowerCase().includes(s));
    }, [q]);
    return <div style={{
      maxWidth: 500
    }}>
        <SearchBar query={q} onQueryChange={setQ} results={results} onSelect={item => alert(\`Selected: \${item.label}\`)} />
        {results.length > 0 && <div style={{
        marginTop: 8,
        fontSize: 12,
        color: 'var(--color-text-secondary)'
      }}>
            Found {results.length} {results.length === 1 ? 'result' : 'results'}
          </div>}
      </div>;
  },
  parameters: {
    docs: {
      description: {
        story: 'Search bar with pre-filled query showing filtered results.'
      }
    }
  }
}`,...c.parameters?.docs?.source}}};const N=["Playground","Basic","WithResults"];export{i as Basic,l as Playground,c as WithResults,N as __namedExportsOrder,H as default};
