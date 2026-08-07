import { createRoot } from 'react-dom/client'
import { httpSessionInspectorDataSource } from './httpDataSource'
import { InspectorApp } from './InspectorApp'

const root = document.getElementById('root')

if (!root) {
  throw new Error('Session Inspector root element is missing')
}

createRoot(root).render(<InspectorApp dataSource={httpSessionInspectorDataSource} />)
