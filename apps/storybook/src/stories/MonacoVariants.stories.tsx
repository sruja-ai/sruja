import { useState } from 'react'
import { MonacoEditor } from '../../../../packages/ui/src/components/MonacoEditor'

export default { title: 'Editor/MonacoEditor Variants' }

export const DarkTheme = () => {
  const [value, setValue] = useState(`specification {
  element person
  element system
}

model {
  user = person "End User"
  app = system "My Application"
  
  user -> app "uses"
}`)
  return (
    <div style={{ height: 300 }}>
      <MonacoEditor value={value} onChange={setValue} theme="vs-dark" />
    </div>
  )
}

export const Tall = () => {
  const [value, setValue] = useState(`specification {
  element person
  element system
  element container
}

model {
  // Architecture definition
  customer = person "Customer"
  
  ecommerce = system "E-Commerce Platform" {
    webApp = container "Web Application"
    api = container "API Service"
  }
  
  customer -> ecommerce.webApp "uses"
}

views {
  view index {
    title "Overview"
    include *
  }
}`)
  return (
    <div style={{ height: 600 }}>
      <MonacoEditor value={value} onChange={setValue} />
    </div>
  )
}

