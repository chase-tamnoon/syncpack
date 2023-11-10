"use strict";(self.webpackChunksite=self.webpackChunksite||[]).push([[9664],{3164:(e,o,t)=>{t.r(o),t.d(o,{assets:()=>l,contentTitle:()=>a,default:()=>h,frontMatter:()=>r,metadata:()=>i,toc:()=>p});var n=t(5893),s=t(1151),c=(t(9960),t(527));const r={title:"Ensure AWS SDK dependencies always have the same version"},a=void 0,i={id:"examples/fix-aws-sdk-version-mismatch",title:"Ensure AWS SDK dependencies always have the same version",description:"",source:"@site/docs/examples/fix-aws-sdk-version-mismatch.mdx",sourceDirName:"examples",slug:"/examples/fix-aws-sdk-version-mismatch",permalink:"/syncpack/examples/fix-aws-sdk-version-mismatch",draft:!1,unlisted:!1,editUrl:"https://github.com/JamieMason/syncpack/tree/main/site/docs/examples/fix-aws-sdk-version-mismatch.mdx",tags:[],version:"current",lastUpdatedBy:"Jamie Mason",lastUpdatedAt:1699631397,formattedLastUpdatedAt:"Nov 10, 2023",frontMatter:{title:"Ensure AWS SDK dependencies always have the same version"},sidebar:"examples",previous:{title:"Ensure that semver ranges for a dependency all match each other",permalink:"/syncpack/examples/ensure-versions-satisfy-same-range"},next:{title:"Fix React Native version mismatch",permalink:"/syncpack/examples/fix-react-native-version-mismatch"}},l={},p=[];function d(e){return(0,n.jsx)(c.ZP,{level:"h2"})}function h(e={}){const{wrapper:o}={...(0,s.a)(),...e.components};return o?(0,n.jsx)(o,{...e,children:(0,n.jsx)(d,{...e})}):d()}},527:(e,o,t)=>{t.d(o,{ZP:()=>h});var n=t(5893),s=t(1151),c=t(9794),r=t(9960),a=t(292),i=t(6217);const l={annotations:c.ds,Code:c.EK},p={staticMediaQuery:"not screen, (max-width: 768px)",lineNumbers:void 0,showCopyButton:!0,themeName:"dracula"};function d(e){const o={code:"code",li:"li",p:"p",strong:"strong",ul:"ul",...(0,s.a)(),...e.components};return l||F("CH",!1),l.Code||F("CH.Code",!0),(0,n.jsxs)(n.Fragment,{children:[(0,n.jsx)("style",{dangerouslySetInnerHTML:{__html:'[data-ch-theme="dracula"] {  --ch-t-colorScheme: dark;--ch-t-foreground: #F8F8F2;--ch-t-background: #282A36;--ch-t-lighter-inlineBackground: #282a36e6;--ch-t-editor-background: #282A36;--ch-t-editor-foreground: #F8F8F2;--ch-t-editor-rangeHighlightBackground: #BD93F915;--ch-t-editor-infoForeground: #3794FF;--ch-t-editor-selectionBackground: #44475A;--ch-t-focusBorder: #6272A4;--ch-t-tab-activeBackground: #282A36;--ch-t-tab-activeForeground: #F8F8F2;--ch-t-tab-inactiveBackground: #21222C;--ch-t-tab-inactiveForeground: #6272A4;--ch-t-tab-border: #191A21;--ch-t-tab-activeBorder: #282A36;--ch-t-editorGroup-border: #BD93F9;--ch-t-editorGroupHeader-tabsBackground: #191A21;--ch-t-editorLineNumber-foreground: #6272A4;--ch-t-input-background: #282A36;--ch-t-input-foreground: #F8F8F2;--ch-t-input-border: #191A21;--ch-t-icon-foreground: #C5C5C5;--ch-t-sideBar-background: #21222C;--ch-t-sideBar-foreground: #F8F8F2;--ch-t-sideBar-border: #21222C;--ch-t-list-activeSelectionBackground: #44475A;--ch-t-list-activeSelectionForeground: #F8F8F2;--ch-t-list-hoverBackground: #44475A75; }'}}),"\n","\n","\n","\n",(0,n.jsxs)(o.p,{children:["Pin all dependencies from ",(0,n.jsx)(r.Z,{to:a.K.awsSdk,children:"@aws-sdk"})," so that they are always identical."]}),"\n",(0,n.jsx)(i.Hx,{level:e.level,children:"1. Add a pinned version group"}),"\n",(0,n.jsxs)(o.ul,{children:["\n",(0,n.jsxs)(o.li,{children:["Match all ",(0,n.jsx)(o.strong,{children:"dependencies"})," whose name starts with ",(0,n.jsx)(o.code,{children:"@aws-sdk/"}),"."]}),"\n",(0,n.jsxs)(o.li,{children:["Mark the version as being pinned to ",(0,n.jsx)(o.strong,{children:"3.272.0"})," in this case."]}),"\n",(0,n.jsxs)(o.li,{children:["Add a ",(0,n.jsx)(o.strong,{children:"label"})," to document the decision/expectation."]}),"\n"]}),"\n",(0,n.jsx)(l.Code,{codeConfig:p,northPanel:{tabs:[""],active:"",heightRatio:1},files:[{name:"",title:'".syncpackrc"',focus:"",code:{lines:[{tokens:[{content:"{",props:{style:{color:"#F8F8F2"}}}]},{tokens:[{content:'  "',props:{style:{color:"#8BE9FE"}}},{content:"versionGroups",props:{style:{color:"#8BE9FD"}}},{content:'"',props:{style:{color:"#8BE9FE"}}},{content:":",props:{style:{color:"#FF79C6"}}},{content:" [",props:{style:{color:"#F8F8F2"}}}]},{tokens:[{content:"    {",props:{style:{color:"#F8F8F2"}}}]},{tokens:[{content:'      "',props:{style:{color:"#8BE9FE"}}},{content:"dependencies",props:{style:{color:"#8BE9FD"}}},{content:'"',props:{style:{color:"#8BE9FE"}}},{content:":",props:{style:{color:"#FF79C6"}}},{content:" [",props:{style:{color:"#F8F8F2"}}},{content:'"',props:{style:{color:"#E9F284"}}},{content:"@aws-sdk/**",props:{style:{color:"#F1FA8C"}}},{content:'"',props:{style:{color:"#E9F284"}}},{content:"],",props:{style:{color:"#F8F8F2"}}}]},{tokens:[{content:'      "',props:{style:{color:"#8BE9FE"}}},{content:"pinVersion",props:{style:{color:"#8BE9FD"}}},{content:'"',props:{style:{color:"#8BE9FE"}}},{content:": ",props:{style:{color:"#FF79C6"}}},{content:'"',props:{style:{color:"#E9F284"}}},{content:"3.272.0",props:{style:{color:"#F1FA8C"}}},{content:'"',props:{style:{color:"#E9F284"}}},{content:",",props:{style:{color:"#F8F8F2"}}}]},{tokens:[{content:'      "',props:{style:{color:"#8BE9FE"}}},{content:"label",props:{style:{color:"#8BE9FD"}}},{content:'"',props:{style:{color:"#8BE9FE"}}},{content:": ",props:{style:{color:"#FF79C6"}}},{content:'"',props:{style:{color:"#E9F284"}}},{content:"AWS SDK Dependencies should all have the same version",props:{style:{color:"#F1FA8C"}}},{content:'"',props:{style:{color:"#E9F284"}}}]},{tokens:[{content:"    }",props:{style:{color:"#F8F8F2"}}}]},{tokens:[{content:"  ]",props:{style:{color:"#F8F8F2"}}}]},{tokens:[{content:"}",props:{style:{color:"#F8F8F2"}}}]}],lang:"json"},annotations:[]}]}),"\n",(0,n.jsx)(i.Hx,{level:e.level,children:"2. Look for mismatches"}),"\n",(0,n.jsxs)(o.p,{children:["Any ",(0,n.jsx)(o.code,{children:"@aws-sdk"})," packages which do not have the expected version can then be found:"]}),"\n",(0,n.jsx)(l.Code,{codeConfig:p,northPanel:{tabs:[""],active:"",heightRatio:1},files:[{name:"",focus:"",code:{lines:[{tokens:[{content:"syncpack ",props:{style:{color:"#F8F8F2"}}},{content:"list-mismatches",props:{style:{color:"#F1FA8C"}}}]}],lang:"bash"},annotations:[]}]}),"\n",(0,n.jsx)(o.p,{children:"And fixed:"}),"\n",(0,n.jsx)(l.Code,{codeConfig:p,northPanel:{tabs:[""],active:"",heightRatio:1},files:[{name:"",focus:"",code:{lines:[{tokens:[{content:"syncpack ",props:{style:{color:"#F8F8F2"}}},{content:"fix-mismatches",props:{style:{color:"#F1FA8C"}}}]}],lang:"bash"},annotations:[]}]})]})}function h(e={}){const{wrapper:o}={...(0,s.a)(),...e.components};return o?(0,n.jsx)(o,{...e,children:(0,n.jsx)(d,{...e})}):d(e)}function F(e,o){throw new Error("Expected "+(o?"component":"object")+" `"+e+"` to be defined: you likely forgot to import, pass, or provide it.")}},6217:(e,o,t)=>{t.d(o,{Hx:()=>s});t(7294);var n=t(5893);function s(e){let{children:o,level:t}=e;return(0,n.jsx)(t,{children:o})}},292:(e,o,t)=>{t.d(o,{K:()=>n});const n={"@types":"https://github.com/DefinitelyTyped/DefinitelyTyped",awsSdk:"https://aws.amazon.com/sdk-for-javascript/",dependencies:"https://docs.npmjs.com/cli/v9/configuring-npm/package-json#dependencies",devDependencies:"https://docs.npmjs.com/cli/v9/configuring-npm/package-json#devDependencies",engines:"https://docs.npmjs.com/cli/v9/configuring-npm/package-json#engines",packageManager:"https://nodejs.org/api/packages.html#packagemanager",version:"https://docs.npmjs.com/cli/v9/configuring-npm/package-json#version",workspaceProtocol:"https://pnpm.io/workspaces#workspace-protocol-workspace"}}}]);