import { useState } from 'react'
import { Listbox } from '../../../../packages/ui/src/components/Listbox'

export default { title: 'Form/Listbox', component: Listbox, tags: ['autodocs'] }

export const Basic = () => {
  const options = [
    { id: 'low', label: 'Low' },
    { id: 'medium', label: 'Medium' },
    { id: 'high', label: 'High' },
  ]
  const [val, setVal] = useState(options[1])
  return <Listbox options={options} value={val} onChange={setVal} label="Priority" />
}

