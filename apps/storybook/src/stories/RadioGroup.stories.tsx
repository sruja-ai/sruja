import { useState } from 'react'
import { RadioGroup } from '../../../../packages/ui/src/components/RadioGroup'

export default { title: 'Form/RadioGroup', component: RadioGroup, tags: ['autodocs'] }

export const Basic = () => {
  const options = [
    { id: 'system', label: 'System', description: 'Top-level node' },
    { id: 'container', label: 'Container', description: 'Service within system' },
    { id: 'datastore', label: 'Datastore', description: 'Database or storage' },
  ]
  const [val, setVal] = useState(options[0])
  return <RadioGroup options={options} value={val} onChange={setVal} label="Node type" />
}

