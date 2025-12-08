import { useState } from 'react'
import { MonacoEditor } from '../../../../packages/ui/src/components/MonacoEditor'

export default { title: 'Editor/MonacoEditor Variants' }

export const DarkTheme = () => {
  const [value, setValue] = useState('person User "User"')
  return (
    <div style={{ height: 300 }}>
      <MonacoEditor value={value} onChange={setValue} theme="vs-dark" />
    </div>
  )
}

export const Tall = () => {
  const [value, setValue] = useState('// taller editor')
  return (
    <div style={{ height: 600 }}>
      <MonacoEditor value={value} onChange={setValue} />
    </div>
  )
}

