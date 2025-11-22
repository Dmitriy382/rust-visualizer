import { Editor } from '@monaco-editor/react';
import { invoke } from '@tauri-apps/api/tauri';
import { Save } from 'lucide-react';
import { useState } from 'react';
import './CodeViewer.css';

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

interface Props {
  module: Module;
  content: string;
}

const CodeViewer = ({ module, content }: Props) => {
  const [code, setCode] = useState(content);
  const [saving, setSaving] = useState(false);
  const [saved, setSaved] = useState(false);

  const handleSave = async () => {
    setSaving(true);
    try {
      await invoke('save_file_content', {
        path: module.path,
        content: code,
      });
      setSaved(true);
      setTimeout(() => setSaved(false), 2000);
    } catch (err) {
      console.error('Failed to save:', err);
    }
    setSaving(false);
  };

  return (
    <div className="code-viewer">
      <div className="code-header">
        <div className="code-title">
          <h3>{module.name}</h3>
          <span className="code-badge">{module.module_type}</span>
        </div>
        <div className="code-path">{module.path}</div>
      </div>

      <div className="code-info">
        <h4>Exported Items ({module.items.length})</h4>
        <div className="items-grid">
          {module.items.slice(0, 10).map((item, idx) => (
            <div key={idx} className="item-chip">
              <span className={`item-visibility ${item.visibility}`}>
                {item.visibility === 'public' ? 'pub' : 'priv'}
              </span>
              <span className="item-type">{item.item_type}</span>
              <span className="item-name">{item.name}</span>
            </div>
          ))}
          {module.items.length > 10 && (
            <div className="item-chip more">
              +{module.items.length - 10} more
            </div>
          )}
        </div>
      </div>

      <div className="editor-container">
        <Editor
          height="100%"
          defaultLanguage="rust"
          theme="vs-dark"
          value={code}
          onChange={(value) => setCode(value || '')}
          options={{
            minimap: { enabled: true },
            fontSize: 14,
            lineNumbers: 'on',
            scrollBeyondLastLine: false,
            automaticLayout: true,
          }}
        />
      </div>
    </div>
  );
};

export default CodeViewer;
