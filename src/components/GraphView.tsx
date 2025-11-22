import { useEffect, useRef } from 'react';
import { Network } from 'vis-network';
import { DataSet } from 'vis-data';
import './GraphView.css';

interface Module {
  id: string;
  name: string;
  path: string;
  module_type: string;
  visibility: string;
  items: any[];
}

interface Relationship {
  from: string;
  to: string;
  rel_type: string;
}

interface ProjectStructure {
  modules: Module[];
  relationships: Relationship[];
  dependencies: any[];
}

interface Props {
  structure: ProjectStructure;
  onModuleClick: (module: Module) => void;
}

const GraphView = ({ structure, onModuleClick }: Props) => {
  const containerRef = useRef<HTMLDivElement>(null);
  const networkRef = useRef<Network | null>(null);

  useEffect(() => {
    if (!containerRef.current) return;

    const getModuleColor = (moduleType: string, visibility: string) => {
      if (moduleType === 'test') return '#ff9800';
      if (moduleType === 'example') return '#2196f3';
      if (moduleType === 'benchmark') return '#9c27b0';
      if (visibility === 'public') return '#4caf50';
      return '#757575';
    };

    const nodes = new DataSet(
      structure.modules.map((module) => ({
    id: module.id,
    label: module.name.split('::').pop() || module.name,
    title: `${module.name}\nType: ${module.module_type}\nVisibility: ${module.visibility}\nItems: ${module.items.length}`,
    color: {
      background: getModuleColor(module.module_type, module.visibility),
      border: '#ffffff',
      highlight: {
        background: '#ff6b35',
        border: '#ffffff',
      },
      hover: {
        background: getModuleColor(module.module_type, module.visibility),
        border: '#ff6b35',
      }
    },
    font: {
      color: '#ffffff',
      size: 16,
      face: 'Inter, system-ui, sans-serif',
      bold: {
        color: '#ffffff',
        size: 16,
      }
    },
    shape: 'box',
    margin: 12,
    borderWidth: 3,
    borderWidthSelected: 4,
    shadow: {
      enabled: true,
      color: 'rgba(0,0,0,0.5)',
      size: 10,
      x: 0,
      y: 4
    },
    data: module,
      }))
    );

    const edges = new DataSet(
      structure.relationships.map((rel, idx) => ({
        id: `edge-${idx}`,
        from: rel.from,
        to: rel.to,
        arrows: 'to',
        color: {
          color: rel.rel_type === 'declares' ? '#4caf50' : '#2196f3',
          highlight: '#ff6b35',
        },
        dashes: rel.rel_type === 'uses',
        title: rel.rel_type,
      }))
    );

    const options = {
       nodes: {
    shape: 'box',
    margin: 12,
    borderWidth: 3,
    borderWidthSelected: 4,
    font: {
      size: 16,
      face: 'Inter, sans-serif',
      color: '#ffffff',
      bold: '600'
    },
    shadow: true,
  },
  edges: {
    width: 3,
    color: {
      color: '#4caf50',
      highlight: '#ff6b35',
      hover: '#66bb6a',
    },
    smooth: {
      type: 'cubicBezier',
      roundness: 0.6,
    },
    shadow: true,
  },
  physics: {
    enabled: true,
    barnesHut: {
      gravitationalConstant: -5000,
      centralGravity: 0.15,
      springLength: 200,
      springConstant: 0.04,
      damping: 0.09,
        },
      },
    };

    const network = new Network(
      containerRef.current,
      { nodes, edges },
      options
    );

    network.on('click', (params) => {
      if (params.nodes.length > 0) {
        const nodeId = params.nodes[0];
        const node = nodes.get(nodeId);
        if (node && node.data) {
          onModuleClick(node.data as Module);
        }
      }
    });

    networkRef.current = network;

    return () => {
      network.destroy();
    };
  }, [structure, onModuleClick]);

  return (
    <div className="graph-container">
      <div ref={containerRef} className="graph" />
      <div className="legend">
        <div className="legend-item">
          <span className="legend-color" style={{ background: '#4caf50' }}></span>
          Public Module
        </div>
        <div className="legend-item">
          <span className="legend-color" style={{ background: '#757575' }}></span>
          Private Module
        </div>
        <div className="legend-item">
          <span className="legend-color" style={{ background: '#ff9800' }}></span>
          Test
        </div>
        <div className="legend-item">
          <span className="legend-color" style={{ background: '#2196f3' }}></span>
          Example
        </div>
      </div>
    </div>
  );
};

export default GraphView;
