import { useState } from 'react'
import { Combobox } from '../../../../packages/ui/src/components/Combobox'

export default { title: 'Form/Combobox', component: Combobox, tags: ['autodocs'] }

export const Basic = () => {
  const [val, setVal] = useState(null)
  const opts = [
    { id: 'api', label: 'API Service' },
    { id: 'db', label: 'Database' },
    { id: 'queue', label: 'Queue' },
  ]
  return <Combobox options={opts} value={val} onChange={setVal} placeholder="Search components" />
}

