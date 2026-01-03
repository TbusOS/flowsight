import { useState } from 'react'
import { invoke } from '@tauri-apps/api/core'

interface AnalysisResult {
  file: string
  functions_count: number
  structs_count: number
  async_handlers_count: number
  entry_points: string[]
}

function App() {
  const [result, setResult] = useState<AnalysisResult | null>(null)
  const [loading, setLoading] = useState(false)
  const [error, setError] = useState<string | null>(null)

  const handleAnalyze = async () => {
    // For now, use a test path - will be replaced with file picker
    const testPath = '/tmp/test.c'
    
    setLoading(true)
    setError(null)
    
    try {
      const analysis = await invoke<AnalysisResult>('analyze_file', { path: testPath })
      setResult(analysis)
    } catch (e) {
      setError(String(e))
    } finally {
      setLoading(false)
    }
  }

  return (
    <div className="app">
      <header className="header">
        <h1>🔭 FlowSight</h1>
        <p>See Your Code Flow</p>
      </header>

      <main className="main">
        <div className="panel">
          <h2>Welcome to FlowSight</h2>
          <p>
            FlowSight helps you understand code execution flow, including:
          </p>
          <ul>
            <li>📊 Function call graphs</li>
            <li>⚙️ Async mechanism tracking (work queues, timers, interrupts)</li>
            <li>🔌 Callback and function pointer resolution</li>
            <li>📦 Data structure relationships</li>
          </ul>

          <button onClick={handleAnalyze} disabled={loading} className="button">
            {loading ? 'Analyzing...' : 'Analyze Test File'}
          </button>

          {error && (
            <div className="error">
              <strong>Error:</strong> {error}
            </div>
          )}

          {result && (
            <div className="result">
              <h3>Analysis Result</h3>
              <p><strong>File:</strong> {result.file}</p>
              <p><strong>Functions:</strong> {result.functions_count}</p>
              <p><strong>Structs:</strong> {result.structs_count}</p>
              <p><strong>Async Handlers:</strong> {result.async_handlers_count}</p>
              <p><strong>Entry Points:</strong> {result.entry_points.join(', ') || 'None'}</p>
            </div>
          )}
        </div>

        <div className="panel placeholder">
          <h2>Code Editor</h2>
          <p>Monaco Editor will be integrated here</p>
        </div>

        <div className="panel placeholder">
          <h2>Flow View</h2>
          <p>Execution flow visualization will appear here</p>
        </div>
      </main>

      <footer className="footer">
        <p>FlowSight v0.1.0 - Made with ❤️ for developers</p>
      </footer>
    </div>
  )
}

export default App

