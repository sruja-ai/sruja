import{S as n}from"./SrujaMonacoEditor-D1OWYQxt.js";import"./preload-helper-PPVm8Dsz.js";import"./iframe-CLHqt8sP.js";import"./MonacoEditor-BeqEQem3.js";const u={title:"Components/SrujaMonacoEditor",component:n,tags:["autodocs"],parameters:{docs:{description:{component:"A Monaco Editor component configured for Sruja language with optional LSP support. Provides syntax highlighting, autocomplete, and language features."}}},argTypes:{value:{control:{type:"text"},description:"Initial editor content"},height:{control:{type:"text"},description:"Height of the editor",table:{defaultValue:{summary:"100%"}}},theme:{control:{type:"select"},options:["vs","vs-dark","hc-black"],description:"Editor theme",table:{defaultValue:{summary:"vs"}}},enableLsp:{control:{type:"boolean"},description:"Whether to enable LSP features",table:{defaultValue:{summary:"true"}}}}},o=`architecture "Example System" {
    system App "My Application" {
        container Web "Web Server" {
            component Auth "Authentication"
            component API "REST API"
        }
        datastore DB "Database"
    }
    
    person User "End User"
    
    User -> App.Web "Uses"
    App.Web.API -> App.DB "Reads/Writes"
}`,e={args:{value:o,height:"400px"}},a={args:{value:o,height:"400px",theme:"vs-dark"}},r={args:{value:o,height:"400px",enableLsp:!1}},t={args:{value:`architecture "Simple" {
    system App "Application"
}`,height:"200px"}},s={args:{value:o,height:"600px"}};e.parameters={...e.parameters,docs:{...e.parameters?.docs,source:{originalSource:`{
  args: {
    value: exampleSruja,
    height: '400px'
  }
}`,...e.parameters?.docs?.source}}};a.parameters={...a.parameters,docs:{...a.parameters?.docs,source:{originalSource:`{
  args: {
    value: exampleSruja,
    height: '400px',
    theme: 'vs-dark'
  }
}`,...a.parameters?.docs?.source}}};r.parameters={...r.parameters,docs:{...r.parameters?.docs,source:{originalSource:`{
  args: {
    value: exampleSruja,
    height: '400px',
    enableLsp: false
  }
}`,...r.parameters?.docs?.source}}};t.parameters={...t.parameters,docs:{...t.parameters?.docs,source:{originalSource:`{
  args: {
    value: \`architecture "Simple" {
    system App "Application"
}\`,
    height: '200px'
  }
}`,...t.parameters?.docs?.source}}};s.parameters={...s.parameters,docs:{...s.parameters?.docs,source:{originalSource:`{
  args: {
    value: exampleSruja,
    height: '600px'
  }
}`,...s.parameters?.docs?.source}}};const m=["Default","DarkTheme","WithoutLsp","Small","Large"];export{a as DarkTheme,e as Default,s as Large,t as Small,r as WithoutLsp,m as __namedExportsOrder,u as default};
