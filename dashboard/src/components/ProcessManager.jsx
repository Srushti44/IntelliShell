/**
 * Process Management Component
 * Lists processes and provides control actions
 */

import { useState } from 'react';
import { Search, RefreshCw, AlertTriangle } from 'lucide-react';
import { useProcesses, useControlProcess } from '../hooks/useProcesses';
import { Card } from './Card';
import { LoadingSpinner } from './LoadingSpinner';
import { ErrorAlert } from './ErrorAlert';
import { formatBytes } from '../lib/utils';

function ProcessRow({ process, onControl, isControlling }) {
  const [showConfirm, setShowConfirm] = useState(false);

  const handleAction = async (action) => {
    setShowConfirm(false);
    await onControl({ pid: process.pid, action });
  };

  return (
    <tr className="hover:bg-gray-50">
      <td className="px-4 py-3 text-sm" style={{ color: 'var(--color-text-muted)', fontFamily: 'var(--font-mono)' }}>{process.pid}</td>
      <td className="px-4 py-3 text-sm truncate max-w-xs" style={{ color: 'var(--ink-dark)' }}>{process.name}</td>
      <td className="px-4 py-3 text-sm" style={{ color: 'var(--color-text-muted)' }}>{process.state}</td>
      <td className="px-4 py-3 text-sm text-right" style={{ color: 'var(--color-text-muted)' }}>
        {formatBytes(process.memory_kb * 1024)}
      </td>
      <td className="px-4 py-3 text-sm text-right" style={{ color: 'var(--color-text-muted)' }}>
        {(process.cpu_percent || 0).toFixed(1)}%
      </td>
      <td className="px-4 py-3 text-sm text-right">
        {!showConfirm ? (
          <button
            onClick={() => setShowConfirm(true)}
            disabled={isControlling}
            className="text-sm disabled:opacity-50 hover:opacity-80"
            style={{ color: 'var(--color-error)' }}
          >
            Terminate
          </button>
        ) : (
          <div className="flex items-center gap-2 justify-end">
            <button
              onClick={() => handleAction('terminate')}
              disabled={isControlling}
              className="text-sm font-semibold disabled:opacity-50 hover:opacity-80"
              style={{ color: 'var(--color-error)' }}
            >
              Confirm
            </button>
            <button
              onClick={() => setShowConfirm(false)}
              disabled={isControlling}
              className="text-sm hover:opacity-80"
              style={{ color: 'var(--color-text-muted)' }}
            >
              Cancel
            </button>
          </div>
        )}
      </td>
    </tr>
  );
}

export function ProcessManager() {
  const [searchTerm, setSearchTerm] = useState('');
  const { data: processes, isLoading, error, refetch } = useProcesses();
  const controlProcess = useControlProcess();

  const filteredProcesses = processes?.filter((proc) =>
    proc.name.toLowerCase().includes(searchTerm.toLowerCase()) ||
    proc.pid.toString().includes(searchTerm)
  ) || [];

  const handleControl = async (params) => {
    try {
      await controlProcess.mutateAsync(params);
    } catch (err) {
      alert(`Failed to control process: ${err.message}`);
    }
  };

  return (
    <Card
      title="Process Manager"
      action={
        <button
          onClick={() => refetch()}
          disabled={isLoading}
          className="flex items-center gap-2 text-sm disabled:opacity-50 hover:opacity-80"
          style={{ color: 'var(--color-primary)' }}
        >
          <RefreshCw className={`w-4 h-4 ${isLoading ? 'animate-spin' : ''}`} />
          Refresh
        </button>
      }
    >
      {error ? (
        <ErrorAlert
          title="Failed to load processes"
          message={error.message}
          onRetry={refetch}
        />
      ) : (
        <>
          {/* Search */}
          <div className="mb-4">
            <div className="relative">
              <Search className="absolute left-3 top-1/2 transform -translate-y-1/2 w-4 h-4" style={{ color: 'var(--color-text-muted)' }} />
              <input
                type="text"
                placeholder="Search by name or PID..."
                value={searchTerm}
                onChange={(e) => setSearchTerm(e.target.value)}
                className="w-full pl-10 pr-4 py-2 border rounded-lg focus:ring-2 focus:ring-[#622B14] focus:border-[#622B14]"
                style={{ borderColor: 'var(--color-border)' }}
              />
            </div>
          </div>

          {/* Warning */}
          <div 
            className="mb-4 border rounded-lg p-3 flex items-start gap-2"
            style={{ backgroundColor: 'var(--color-warning-bg)', borderColor: 'var(--color-warning)' }}
          >
            <AlertTriangle className="w-4 h-4 flex-shrink-0 mt-0.5" style={{ color: 'var(--color-warning)' }} />
            <p className="text-xs" style={{ color: 'var(--ink-dark)' }}>
              Terminating processes can cause data loss or system instability. Use with caution.
            </p>
          </div>

          {/* Process Table */}
          {isLoading ? (
            <div className="py-8">
              <LoadingSpinner text="Loading processes..." />
            </div>
          ) : filteredProcesses.length === 0 ? (
            <div className="text-center py-8" style={{ color: 'var(--color-text-muted)' }}>
              {searchTerm ? 'No processes match your search' : 'No processes found'}
            </div>
          ) : (
            <div className="overflow-x-auto">
              <table className="min-w-full divide-y divide-gray-200">
                <thead style={{ backgroundColor: 'var(--color-bg-subtle)' }}>
                  <tr>
                    <th className="px-4 py-3 text-left text-xs font-medium uppercase" style={{ color: 'var(--color-text-muted)' }}>PID</th>
                    <th className="px-4 py-3 text-left text-xs font-medium uppercase" style={{ color: 'var(--color-text-muted)' }}>Name</th>
                    <th className="px-4 py-3 text-left text-xs font-medium uppercase" style={{ color: 'var(--color-text-muted)' }}>State</th>
                    <th className="px-4 py-3 text-right text-xs font-medium uppercase" style={{ color: 'var(--color-text-muted)' }}>Memory</th>
                    <th className="px-4 py-3 text-right text-xs font-medium uppercase" style={{ color: 'var(--color-text-muted)' }}>CPU</th>
                    <th className="px-4 py-3 text-right text-xs font-medium uppercase" style={{ color: 'var(--color-text-muted)' }}>Actions</th>
                  </tr>
                </thead>
                <tbody className="divide-y divide-gray-200" style={{ backgroundColor: 'var(--offwhite-main)' }}>
                  {filteredProcesses.map((process) => (
                    <ProcessRow
                      key={process.pid}
                      process={process}
                      onControl={handleControl}
                      isControlling={controlProcess.isPending}
                    />
                  ))}
                </tbody>
              </table>
            </div>
          )}

          {/* Count */}
          <div className="mt-4 text-sm" style={{ color: 'var(--color-text-muted)' }}>
            Showing {filteredProcesses.length} of {processes?.length || 0} processes
          </div>
        </>
      )}
    </Card>
  );
}
