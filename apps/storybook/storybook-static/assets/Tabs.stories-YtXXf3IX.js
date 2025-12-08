import{a as d,R as $,j as r}from"./iframe-CLHqt8sP.js";import{$ as le,a as we}from"./useFocusRing-4vPDgUpM.js";import{w as Se,e as je}from"./use-resolve-button-type-DhPqIc8n.js";import{Y as W,y as G,n as ee,o as P,V as de,K as M,s as U,A as se,u as L,t as Te}from"./use-sync-refs-v6ctzwoA.js";import{n as q,o as h,e as Ce}from"./keyboard-DOqIamXD.js";import{f as Ee}from"./transition-Bjg5DtQF.js";import{f as ce,s as Pe}from"./hidden-CeoxBUgT.js";import{v as A,T as w,A as _,G as F}from"./focus-management-DE1YMz3g.js";import{C as J}from"./Card-BHe2Chf6.js";import{B as K}from"./Badge-DGs5HKmX.js";import"./preload-helper-PPVm8Dsz.js";import"./open-closed-B--VY9yg.js";function Ie({onFocus:e}){let[t,o]=d.useState(!0),n=Ee();return t?$.createElement(ce,{as:"button",type:"button",features:Pe.Focusable,onFocus:s=>{s.preventDefault();let a,i=50;function c(){if(i--<=0){a&&cancelAnimationFrame(a);return}if(e()){if(cancelAnimationFrame(a),!n.current)return;o(!1);return}a=requestAnimationFrame(c)}a=requestAnimationFrame(c)}}):null}const ue=d.createContext(null);function ke(){return{groups:new Map,get(e,t){var o;let n=this.groups.get(e);n||(n=new Map,this.groups.set(e,n));let s=(o=n.get(t))!=null?o:0;n.set(t,s+1);let a=Array.from(n.keys()).indexOf(t);function i(){let c=n.get(t);c>1?n.set(t,c-1):n.delete(t)}return[a,i]}}}function Ae({children:e}){let t=d.useRef(ke());return d.createElement(ue.Provider,{value:t},e)}function pe(e){let t=d.useContext(ue);if(!t)throw new Error("You must wrap your component in a <StableCollection>");let o=d.useId(),[n,s]=t.current.get(e,o);return d.useEffect(()=>s,[]),n}var $e=(e=>(e[e.Forwards=0]="Forwards",e[e.Backwards=1]="Backwards",e))($e||{}),De=(e=>(e[e.Less=-1]="Less",e[e.Equal=0]="Equal",e[e.Greater=1]="Greater",e))(De||{}),Ve=(e=>(e[e.SetSelectedIndex=0]="SetSelectedIndex",e[e.RegisterTab=1]="RegisterTab",e[e.UnregisterTab=2]="UnregisterTab",e[e.RegisterPanel=3]="RegisterPanel",e[e.UnregisterPanel=4]="UnregisterPanel",e))(Ve||{});let Re={0(e,t){var o;let n=F(e.tabs,u=>u.current),s=F(e.panels,u=>u.current),a=n.filter(u=>{var b;return!((b=u.current)!=null&&b.hasAttribute("disabled"))}),i={...e,tabs:n,panels:s};if(t.index<0||t.index>n.length-1){let u=L(Math.sign(t.index-e.selectedIndex),{[-1]:()=>1,0:()=>L(Math.sign(t.index),{[-1]:()=>0,0:()=>0,1:()=>1}),1:()=>0});if(a.length===0)return i;let b=L(u,{0:()=>n.indexOf(a[0]),1:()=>n.indexOf(a[a.length-1])});return{...i,selectedIndex:b===-1?e.selectedIndex:b}}let c=n.slice(0,t.index),S=[...n.slice(t.index),...c].find(u=>a.includes(u));if(!S)return i;let x=(o=n.indexOf(S))!=null?o:e.selectedIndex;return x===-1&&(x=e.selectedIndex),{...i,selectedIndex:x}},1(e,t){if(e.tabs.includes(t.tab))return e;let o=e.tabs[e.selectedIndex],n=F([...e.tabs,t.tab],a=>a.current),s=e.selectedIndex;return e.info.current.isControlled||(s=n.indexOf(o),s===-1&&(s=e.selectedIndex)),{...e,tabs:n,selectedIndex:s}},2(e,t){return{...e,tabs:e.tabs.filter(o=>o!==t.tab)}},3(e,t){return e.panels.includes(t.panel)?e:{...e,panels:F([...e.panels,t.panel],o=>o.current)}},4(e,t){return{...e,panels:e.panels.filter(o=>o!==t.panel)}}},ne=d.createContext(null);ne.displayName="TabsDataContext";function D(e){let t=d.useContext(ne);if(t===null){let o=new Error(`<${e} /> is missing a parent <Tab.Group /> component.`);throw Error.captureStackTrace&&Error.captureStackTrace(o,D),o}return t}let oe=d.createContext(null);oe.displayName="TabsActionsContext";function ae(e){let t=d.useContext(oe);if(t===null){let o=new Error(`<${e} /> is missing a parent <Tab.Group /> component.`);throw Error.captureStackTrace&&Error.captureStackTrace(o,ae),o}return t}function ze(e,t){return L(t.type,Re,e,t)}let Ne="div";function Oe(e,t){let{defaultIndex:o=0,vertical:n=!1,manual:s=!1,onChange:a,selectedIndex:i=null,...c}=e;const S=n?"vertical":"horizontal",x=s?"manual":"auto";let u=i!==null,b=U({isControlled:u}),I=G(t),[f,p]=d.useReducer(ze,{info:b,selectedIndex:i??o,tabs:[],panels:[]}),V=q({selectedIndex:f.selectedIndex}),R=U(a||(()=>{})),T=U(f.tabs),v=d.useMemo(()=>({orientation:S,activation:x,...f}),[S,x,f]),z=P(g=>(p({type:1,tab:g}),()=>p({type:2,tab:g}))),N=P(g=>(p({type:3,panel:g}),()=>p({type:4,panel:g}))),m=P(g=>{y.current!==g&&R.current(g),u||p({type:0,index:g})}),y=U(u?e.selectedIndex:f.selectedIndex),E=d.useMemo(()=>({registerTab:z,registerPanel:N,change:m}),[]);ee(()=>{p({type:0,index:i??o})},[i]),ee(()=>{if(y.current===void 0||f.tabs.length<=0)return;let g=F(f.tabs,C=>C.current);g.some((C,k)=>f.tabs[k]!==C)&&m(g.indexOf(f.tabs[y.current]))});let te={ref:I},O=M();return $.createElement(Ae,null,$.createElement(oe.Provider,{value:E},$.createElement(ne.Provider,{value:v},v.tabs.length<=0&&$.createElement(Ie,{onFocus:()=>{var g,C;for(let k of T.current)if(((g=k.current)==null?void 0:g.tabIndex)===0)return(C=k.current)==null||C.focus(),!0;return!1}}),O({ourProps:te,theirProps:c,slot:V,defaultTag:Ne,name:"Tabs"}))))}let Be="div";function Fe(e,t){let{orientation:o,selectedIndex:n}=D("Tab.List"),s=G(t),a=q({selectedIndex:n}),i=e,c={ref:s,role:"tablist","aria-orientation":o};return M()({ourProps:c,theirProps:i,slot:a,defaultTag:Be,name:"Tabs.List"})}let Le="button";function We(e,t){var o,n;let s=d.useId(),{id:a=`headlessui-tabs-tab-${s}`,disabled:i=!1,autoFocus:c=!1,...S}=e,{orientation:x,activation:u,selectedIndex:b,tabs:I,panels:f}=D("Tab"),p=ae("Tab"),V=D("Tab"),[R,T]=d.useState(null),v=d.useRef(null),z=G(v,t,T);ee(()=>p.registerTab(v),[p,v]);let N=pe("tabs"),m=I.indexOf(v);m===-1&&(m=N);let y=m===b,E=P(l=>{let j=l();if(j===_.Success&&u==="auto"){let re=Ce(v.current),ie=V.tabs.findIndex(he=>he.current===re);ie!==-1&&p.change(ie)}return j}),te=P(l=>{let j=I.map(re=>re.current).filter(Boolean);if(l.key===h.Space||l.key===h.Enter){l.preventDefault(),l.stopPropagation(),p.change(m);return}switch(l.key){case h.Home:case h.PageUp:return l.preventDefault(),l.stopPropagation(),E(()=>A(j,w.First));case h.End:case h.PageDown:return l.preventDefault(),l.stopPropagation(),E(()=>A(j,w.Last))}if(E(()=>L(x,{vertical(){return l.key===h.ArrowUp?A(j,w.Previous|w.WrapAround):l.key===h.ArrowDown?A(j,w.Next|w.WrapAround):_.Error},horizontal(){return l.key===h.ArrowLeft?A(j,w.Previous|w.WrapAround):l.key===h.ArrowRight?A(j,w.Next|w.WrapAround):_.Error}}))===_.Success)return l.preventDefault()}),O=d.useRef(!1),g=P(()=>{var l;O.current||(O.current=!0,(l=v.current)==null||l.focus({preventScroll:!0}),p.change(m),Te(()=>{O.current=!1}))}),C=P(l=>{l.preventDefault()}),{isFocusVisible:k,focusProps:ge}=le({autoFocus:c}),{isHovered:fe,hoverProps:ve}=we({isDisabled:i}),{pressed:xe,pressProps:be}=Se({disabled:i}),me=q({selected:y,hover:fe,active:xe,focus:k,autofocus:c,disabled:i}),ye=de({ref:z,onKeyDown:te,onMouseDown:C,onClick:g,id:a,role:"tab",type:je(e,R),"aria-controls":(n=(o=f[m])==null?void 0:o.current)==null?void 0:n.id,"aria-selected":y,tabIndex:y?0:-1,disabled:i||void 0,autoFocus:c},ge,ve,be);return M()({ourProps:ye,theirProps:S,slot:me,defaultTag:Le,name:"Tabs.Tab"})}let Ge="div";function Me(e,t){let{selectedIndex:o}=D("Tab.Panels"),n=G(t),s=q({selectedIndex:o}),a=e,i={ref:n};return M()({ourProps:i,theirProps:a,slot:s,defaultTag:Ge,name:"Tabs.Panels"})}let qe="div",He=se.RenderStrategy|se.Static;function Ue(e,t){var o,n,s,a;let i=d.useId(),{id:c=`headlessui-tabs-panel-${i}`,tabIndex:S=0,...x}=e,{selectedIndex:u,tabs:b,panels:I}=D("Tab.Panel"),f=ae("Tab.Panel"),p=d.useRef(null),V=G(p,t);ee(()=>f.registerPanel(p),[f,p]);let R=pe("panels"),T=I.indexOf(p);T===-1&&(T=R);let v=T===u,{isFocusVisible:z,focusProps:N}=le(),m=q({selected:v,focus:z}),y=de({ref:V,id:c,role:"tabpanel","aria-labelledby":(n=(o=b[T])==null?void 0:o.current)==null?void 0:n.id,tabIndex:v?S:-1},N),E=M();return!v&&((s=x.unmount)==null||s)&&!((a=x.static)!=null&&a)?$.createElement(ce,{"aria-hidden":"true",...y}):E({ourProps:y,theirProps:x,slot:m,defaultTag:qe,features:He,visible:v,name:"Tabs.Panel"})}let _e=W(We),Je=W(Oe),Ke=W(Fe),Ye=W(Me),Xe=W(Ue),B=Object.assign(_e,{Group:Je,List:Ke,Panels:Ye,Panel:Xe});function H({tabs:e,defaultIndex:t=0,onChange:o}){return r.jsxs(B.Group,{defaultIndex:t,onChange:o,children:[r.jsx(B.List,{className:"flex gap-2 border-b border-[var(--color-border)]",children:e.map(n=>r.jsx(B,{as:d.Fragment,children:({selected:s})=>r.jsx("button",{className:["px-3.5 py-2 text-sm rounded-t-md","focus:outline-none focus-visible:ring-2 focus-visible:ring-offset-2 focus-visible:ring-[var(--color-primary)]",s?"bg-[var(--color-background)] text-[var(--color-text-primary)] border border-b-transparent border-[var(--color-border)]":"text-[var(--color-text-secondary)] hover:bg-[var(--color-surface)]"].join(" "),children:n.label})},n.id))}),r.jsx(B.Panels,{className:"mt-3",children:e.map(n=>r.jsx(B.Panel,{className:"focus:outline-none focus-visible:ring-2 focus-visible:ring-[var(--color-primary)] focus-visible:ring-offset-2 rounded-md",children:n.content},n.id))})]})}H.__docgenInfo={description:"",methods:[],displayName:"Tabs",props:{tabs:{required:!0,tsType:{name:"Array",elements:[{name:"signature",type:"object",raw:"{ id: string; label: string; content: any }",signature:{properties:[{key:"id",value:{name:"string",required:!0}},{key:"label",value:{name:"string",required:!0}},{key:"content",value:{name:"any",required:!0}}]}}],raw:"Array<{ id: string; label: string; content: any }>"},description:""},defaultIndex:{required:!1,tsType:{name:"number"},description:"",defaultValue:{value:"0",computed:!1}},onChange:{required:!1,tsType:{name:"signature",type:"function",raw:"(index: number) => void",signature:{arguments:[{type:{name:"number"},name:"index"}],return:{name:"void"}}},description:""}}};const ct={title:"Components/Tabs",component:H,tags:["autodocs"],parameters:{layout:"centered",docs:{description:{component:"Tab navigation component for organizing content into sections. Used in Sruja Studio for switching between editor, viewer, and documentation views."}}},argTypes:{tabs:{description:"Array of tab definitions with id, label, and content"}}},Y={args:{tabs:[{id:"editor",label:"Editor",content:r.jsx("div",{style:{padding:16},children:"Editor content"})},{id:"viewer",label:"Viewer",content:r.jsx("div",{style:{padding:16},children:"Viewer content"})},{id:"docs",label:"Documentation",content:r.jsx("div",{style:{padding:16},children:"Documentation content"})}]}},X={render:()=>r.jsx(H,{tabs:[{id:"editor",label:"Editor",content:r.jsx("div",{style:{padding:24,backgroundColor:"var(--color-surface)",minHeight:200,display:"flex",alignItems:"center",justifyContent:"center",color:"var(--color-text-secondary)"},children:r.jsxs("div",{style:{textAlign:"center"},children:[r.jsx("div",{style:{fontSize:18,fontWeight:600,marginBottom:8},children:"DSL Editor"}),r.jsx("div",{style:{fontSize:14},children:"Monaco editor with syntax highlighting"})]})})},{id:"split",label:"Split View",content:r.jsx("div",{style:{padding:24,backgroundColor:"var(--color-surface)",minHeight:200,display:"flex",alignItems:"center",justifyContent:"center",color:"var(--color-text-secondary)"},children:r.jsxs("div",{style:{textAlign:"center"},children:[r.jsx("div",{style:{fontSize:18,fontWeight:600,marginBottom:8},children:"Split View"}),r.jsx("div",{style:{fontSize:14},children:"Editor and diagram side by side"})]})})},{id:"viewer",label:"Viewer",content:r.jsx("div",{style:{padding:24,backgroundColor:"var(--color-surface)",minHeight:200,display:"flex",alignItems:"center",justifyContent:"center",color:"var(--color-text-secondary)"},children:r.jsxs("div",{style:{textAlign:"center"},children:[r.jsx("div",{style:{fontSize:18,fontWeight:600,marginBottom:8},children:"Diagram Viewer"}),r.jsx("div",{style:{fontSize:14},children:"Interactive architecture diagram"})]})})}]}),parameters:{docs:{description:{story:"Tabs for switching between different view modes in Studio."}}}},Q={render:()=>r.jsx(H,{tabs:[{id:"overview",label:"Overview",content:r.jsxs("div",{style:{padding:24},children:[r.jsx("h3",{style:{marginTop:0},children:"Architecture Overview"}),r.jsxs("div",{style:{display:"grid",gridTemplateColumns:"repeat(auto-fit, minmax(200px, 1fr))",gap:16},children:[r.jsx(J,{title:"Systems",footer:r.jsx(K,{color:"brand",children:"3"}),children:r.jsx("p",{style:{fontSize:14,color:"var(--color-text-secondary)",margin:0},children:"Total systems"})}),r.jsx(J,{title:"Containers",footer:r.jsx(K,{children:"12"}),children:r.jsx("p",{style:{fontSize:14,color:"var(--color-text-secondary)",margin:0},children:"Container components"})}),r.jsx(J,{title:"Relations",footer:r.jsx(K,{color:"info",children:"24"}),children:r.jsx("p",{style:{fontSize:14,color:"var(--color-text-secondary)",margin:0},children:"Connections"})})]})]})},{id:"validation",label:"Validation",content:r.jsxs("div",{style:{padding:24},children:[r.jsx("h3",{style:{marginTop:0},children:"Validation Results"}),r.jsx(J,{title:"Status",footer:r.jsx(K,{color:"success",children:"Valid"}),children:r.jsx("p",{style:{fontSize:14,color:"var(--color-text-secondary)",margin:0},children:"No errors or warnings found in the architecture definition."})})]})},{id:"export",label:"Export",content:r.jsxs("div",{style:{padding:24},children:[r.jsx("h3",{style:{marginTop:0},children:"Export Options"}),r.jsxs("div",{style:{display:"flex",flexDirection:"column",gap:12},children:[r.jsx("button",{style:{padding:"12px 16px",borderRadius:6,border:"1px solid var(--color-border)",backgroundColor:"var(--color-background)",cursor:"pointer",textAlign:"left"},children:"Export as PNG"}),r.jsx("button",{style:{padding:"12px 16px",borderRadius:6,border:"1px solid var(--color-border)",backgroundColor:"var(--color-background)",cursor:"pointer",textAlign:"left"},children:"Export as SVG"}),r.jsx("button",{style:{padding:"12px 16px",borderRadius:6,border:"1px solid var(--color-border)",backgroundColor:"var(--color-background)",cursor:"pointer",textAlign:"left"},children:"Export as JSON"})]})]})}]}),parameters:{docs:{description:{story:"Tabs with rich content sections for architecture management."}}}},Z={render:()=>r.jsx("div",{className:"bg-[var(--color-background)] border border-[var(--color-border)] rounded-lg shadow-sm p-5 max-w-2xl",children:r.jsx(H,{tabs:[{id:"overview",label:"Overview",content:r.jsx("div",{className:"p-4 text-[var(--color-text-secondary)]",children:"Overview content"})},{id:"diagram",label:"Diagram",content:r.jsx("div",{className:"p-4 text-[var(--color-text-secondary)]",children:"Diagram content"})},{id:"export",label:"Export",content:r.jsx("div",{className:"p-4 text-[var(--color-text-secondary)]",children:"Export options"})}]})})};Y.parameters={...Y.parameters,docs:{...Y.parameters?.docs,source:{originalSource:`{
  args: {
    tabs: [{
      id: 'editor',
      label: 'Editor',
      content: <div style={{
        padding: 16
      }}>Editor content</div>
    }, {
      id: 'viewer',
      label: 'Viewer',
      content: <div style={{
        padding: 16
      }}>Viewer content</div>
    }, {
      id: 'docs',
      label: 'Documentation',
      content: <div style={{
        padding: 16
      }}>Documentation content</div>
    }]
  }
}`,...Y.parameters?.docs?.source}}};X.parameters={...X.parameters,docs:{...X.parameters?.docs,source:{originalSource:`{
  render: () => <Tabs tabs={[{
    id: 'editor',
    label: 'Editor',
    content: <div style={{
      padding: 24,
      backgroundColor: 'var(--color-surface)',
      minHeight: 200,
      display: 'flex',
      alignItems: 'center',
      justifyContent: 'center',
      color: 'var(--color-text-secondary)'
    }}>
              <div style={{
        textAlign: 'center'
      }}>
                <div style={{
          fontSize: 18,
          fontWeight: 600,
          marginBottom: 8
        }}>DSL Editor</div>
                <div style={{
          fontSize: 14
        }}>Monaco editor with syntax highlighting</div>
              </div>
            </div>
  }, {
    id: 'split',
    label: 'Split View',
    content: <div style={{
      padding: 24,
      backgroundColor: 'var(--color-surface)',
      minHeight: 200,
      display: 'flex',
      alignItems: 'center',
      justifyContent: 'center',
      color: 'var(--color-text-secondary)'
    }}>
              <div style={{
        textAlign: 'center'
      }}>
                <div style={{
          fontSize: 18,
          fontWeight: 600,
          marginBottom: 8
        }}>Split View</div>
                <div style={{
          fontSize: 14
        }}>Editor and diagram side by side</div>
              </div>
            </div>
  }, {
    id: 'viewer',
    label: 'Viewer',
    content: <div style={{
      padding: 24,
      backgroundColor: 'var(--color-surface)',
      minHeight: 200,
      display: 'flex',
      alignItems: 'center',
      justifyContent: 'center',
      color: 'var(--color-text-secondary)'
    }}>
              <div style={{
        textAlign: 'center'
      }}>
                <div style={{
          fontSize: 18,
          fontWeight: 600,
          marginBottom: 8
        }}>Diagram Viewer</div>
                <div style={{
          fontSize: 14
        }}>Interactive architecture diagram</div>
              </div>
            </div>
  }]} />,
  parameters: {
    docs: {
      description: {
        story: 'Tabs for switching between different view modes in Studio.'
      }
    }
  }
}`,...X.parameters?.docs?.source}}};Q.parameters={...Q.parameters,docs:{...Q.parameters?.docs,source:{originalSource:`{
  render: () => <Tabs tabs={[{
    id: 'overview',
    label: 'Overview',
    content: <div style={{
      padding: 24
    }}>
              <h3 style={{
        marginTop: 0
      }}>Architecture Overview</h3>
              <div style={{
        display: 'grid',
        gridTemplateColumns: 'repeat(auto-fit, minmax(200px, 1fr))',
        gap: 16
      }}>
                <Card title="Systems" footer={<Badge color="brand">3</Badge>}>
                  <p style={{
            fontSize: 14,
            color: 'var(--color-text-secondary)',
            margin: 0
          }}>Total systems</p>
                </Card>
                <Card title="Containers" footer={<Badge>12</Badge>}>
                  <p style={{
            fontSize: 14,
            color: 'var(--color-text-secondary)',
            margin: 0
          }}>Container components</p>
                </Card>
                <Card title="Relations" footer={<Badge color="info">24</Badge>}>
                  <p style={{
            fontSize: 14,
            color: 'var(--color-text-secondary)',
            margin: 0
          }}>Connections</p>
                </Card>
              </div>
            </div>
  }, {
    id: 'validation',
    label: 'Validation',
    content: <div style={{
      padding: 24
    }}>
              <h3 style={{
        marginTop: 0
      }}>Validation Results</h3>
              <Card title="Status" footer={<Badge color="success">Valid</Badge>}>
                <p style={{
          fontSize: 14,
          color: 'var(--color-text-secondary)',
          margin: 0
        }}>
                  No errors or warnings found in the architecture definition.
                </p>
              </Card>
            </div>
  }, {
    id: 'export',
    label: 'Export',
    content: <div style={{
      padding: 24
    }}>
              <h3 style={{
        marginTop: 0
      }}>Export Options</h3>
              <div style={{
        display: 'flex',
        flexDirection: 'column',
        gap: 12
      }}>
                <button style={{
          padding: '12px 16px',
          borderRadius: 6,
          border: '1px solid var(--color-border)',
          backgroundColor: 'var(--color-background)',
          cursor: 'pointer',
          textAlign: 'left'
        }}>
                  Export as PNG
                </button>
                <button style={{
          padding: '12px 16px',
          borderRadius: 6,
          border: '1px solid var(--color-border)',
          backgroundColor: 'var(--color-background)',
          cursor: 'pointer',
          textAlign: 'left'
        }}>
                  Export as SVG
                </button>
                <button style={{
          padding: '12px 16px',
          borderRadius: 6,
          border: '1px solid var(--color-border)',
          backgroundColor: 'var(--color-background)',
          cursor: 'pointer',
          textAlign: 'left'
        }}>
                  Export as JSON
                </button>
              </div>
            </div>
  }]} />,
  parameters: {
    docs: {
      description: {
        story: 'Tabs with rich content sections for architecture management.'
      }
    }
  }
}`,...Q.parameters?.docs?.source}}};Z.parameters={...Z.parameters,docs:{...Z.parameters?.docs,source:{originalSource:`{
  render: () => <div className="bg-[var(--color-background)] border border-[var(--color-border)] rounded-lg shadow-sm p-5 max-w-2xl">
      <Tabs tabs={[{
      id: 'overview',
      label: 'Overview',
      content: <div className="p-4 text-[var(--color-text-secondary)]">Overview content</div>
    }, {
      id: 'diagram',
      label: 'Diagram',
      content: <div className="p-4 text-[var(--color-text-secondary)]">Diagram content</div>
    }, {
      id: 'export',
      label: 'Export',
      content: <div className="p-4 text-[var(--color-text-secondary)]">Export options</div>
    }]} />
    </div>
}`,...Z.parameters?.docs?.source}}};const ut=["Playground","ViewSwitcher","WithContent","Showcase"];export{Y as Playground,Z as Showcase,X as ViewSwitcher,Q as WithContent,ut as __namedExportsOrder,ct as default};
