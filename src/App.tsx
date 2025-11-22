import { useState } from 'react';
import { open } from '@tauri-apps/api/dialog';
import { invoke } from '@tauri-apps/api/tauri';
import { FolderOpen, Loader2, AlertCircle } from 'lucide-react';
import GraphView from './components/GraphView';
import CodeViewer from './components/CodeViewer';
import './App.css';

interface ProjectStructure {
  root_path: string;
  modules: Module[];
  dependencies: Dependency[];
  relationships: Relationship[];
}

interface Module {
  id: string;
  name: string;
  path: string;
  module_type: string;
  visibility: string;
  items: Item[];
}

interface Item {
  name: string;
  item_type: string;
  visibility: string;
}

interface Dependency {
  name: string;
  version: string;
  dep_type: string;
}

interface Relationship {
  from: string;
  to: string;
  rel_type: string;
}

function App() {
  const [structure, setStructure] = useState<ProjectStructure | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [loadingFile, setLoadingFile] = useState(false);
  const [selectedModule, setSelectedModule] = useState<Module | null>(null);
  const [fileContent, setFileContent] = useState<string>('');
  const [search, setSearch] = useState('');
  const [problems, setProblems] = useState<any>(null);

  const handleSelectFolder = async () => {
    try {
      const selected = await open({
        directory: true,
        multiple: false,
      });

      if (selected && typeof selected === 'string') {
        setLoading(true);
        setError(null);
        
        const result = await invoke<ProjectStructure>('analyze_project', {
          path: selected,
        });
        
        setStructure(result);
        setLoading(false);
        const probs = await invoke('analyze_problems', { structure: result });
        setProblems(probs);
      }
    } catch (err) {
      setError(err as string);
      setLoading(false);
    }
  };

  const handleModuleClick = async (module: Module) => {
    setSelectedModule(module);
    setLoadingFile(true);
    try {
      const content = await invoke<string>('read_file_content', {
        path: module.path,
      });
      setFileContent(content);
    } catch (err) {
      console.error('Failed to read file:', err);
      setFileContent('// Failed to load file content');
    }
    setLoadingFile(false);
  };
  
  const generateDocs = async () => {
    if (!structure) return;
  
    try {
      const path = await invoke<string>('generate_documentation', {
        structure: structure,
      });
      alert(`✅ Documentation generated!\n\nSaved to: ${path}`);
    } catch (err) {
      alert('❌ Failed to generate docs: ' + err);
    }
  };

  return (
    <div className="app">
      <header className="header">
        <h1>Rust Project Visualizer</h1>

        {structure && (
          <input
            type="text"
            placeholder="Search modules..."
            value={search}
            onChange={(e) => setSearch(e.target.value)}
            style={{
                padding: '0.6rem 1rem',
                background: '#333',
                border: '1px solid #555',
                color: '#fff',
                borderRadius: '8px',
                width: '300px'
            }}
          />
        )}

        <button onClick={handleSelectFolder} className="btn-primary" disabled={loading}>
          {loading ? (
            <>
              <Loader2 className="icon spinning" />
              Analyzing...
            </>
          ) : (
            <>
              <FolderOpen className="icon" />
              Open Project
            </>
          )}
        </button>
        {structure && (
          <button 
            onClick={generateDocs} 
            style={{
                padding: '0.7rem 1.4rem',
                background: 'linear-gradient(135deg, #4caf50 0%, #66bb6a 100%)',
                color: 'white',
                border: 'none',
                borderRadius: '8px',
                cursor: 'pointer',
                fontWeight: '600',
                boxShadow: '0 4px 15px rgba(76, 175, 80, 0.4)',
                display: 'flex',
                alignItems: 'center',
                gap: '0.5rem'
              }}
            >
                Generate Docs
              </button>
            )}
      </header>

      {error && (
        <div className="error-banner">
          <AlertCircle className="icon" />
          <span>{error}</span>
        </div>
      )}
      
      {structure && (
        <div style={{padding: '1rem 2rem', background: 'rgba(36, 36, 36, 0.8)', borderBottom: '1px solid #333', display: 'flex', gap: '2rem', fontSize: '0.9rem'}}>
          <span>Modules: <strong>{structure.modules.length}</strong></span>
          <span>Dependencies: <strong>{structure.dependencies.length}</strong></span>
          <span>Public: <strong>{structure.modules.filter(m => m.visibility === 'public').length}</strong></span>
        </div>
)}

      <main className="content">
        {structure ? (
          <>
            <div className="graph-panel">
              <GraphView
               structure={{
                  ...structure,
                  modules: structure.modules.filter(m =>
                    m.name.toLowerCase().includes(search.toLowerCase())
                  ),
                  relationships: structure.relationships.filter(r =>
                    structure.modules.some(m => 
                    (m.id === r.from || m.id === r.to) &&
                    m.name.toLowerCase().includes(search.toLowerCase())
                  )
                )
            }}
            onModuleClick={handleModuleClick}              
          />
            </div>
            
            {selectedModule && (
              <div className="problems-panel">
                <h2>⚠️ Problems</h2>
    
                {problems?.cycles?.length > 0 && (
                  <div className="problem-section">
                    <h3>🔄 Circular Dependencies</h3>
                    {problems.cycles.map((cycle: string[], i: number) => (
                        <div key={i} className="problem-item error">
                            {cycle.join(' → ')}
                        </div>
                      ))}
                    </div>
                  )}
    
                {problems?.unused_modules?.length > 0 && (
                    <div className="problem-section">
                        <h3>Unused Modules</h3>
                        {problems.unused_modules.map((mod: string, i: number) => (
                          <div key={i} className="problem-item warning">{mod}</div>
                      ))}
                    </div>
                  )}
                </div>
            )}
          </>
        ) : (
          <div className="empty-state">
            <FolderOpen size={64} className="icon-large" />
            <h2>No Project Loaded</h2>
            <p>Select a Rust project folder to visualize its structure</p>
          </div>
        )}
      </main>
    </div>
  );
}

export default App;
