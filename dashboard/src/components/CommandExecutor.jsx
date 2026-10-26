/**
 * Command Executor Component
 * Handles direct command execution with risk assessment
 */

import { useState } from 'react';
import { Terminal, Play, AlertTriangle, CheckCircle, XCircle } from 'lucide-react';
import { useMutation } from '@tanstack/react-query';
import { rustApi } from '../lib/api';
import { Card } from './Card';
import { StatusBadge } from './StatusBadge';
import { LoadingSpinner } from './LoadingSpinner';
import { parseCommand, formatDuration } from '../lib/utils';

export function CommandExecutor({ initialCommand = '' }) {
  const [commandInput, setCommandInput] = useState(initialCommand);
  const [riskLevel, setRiskLevel] = useState('LOW');
  const [result, setResult] = useState(null);

  const executeMutation = useMutation({
    mutationFn: ({ command, args, riskLevel }) =>
      rustApi.executeCommand(command, args, riskLevel),
    onSuccess: (data) => {
      setResult({ success: true, data });
    },
    onError: (error) => {
      setResult({ success: false, error });
    },
  });

  const handleExecute = () => {
    if (!commandInput.trim()) return;

    const { command, args } = parseCommand(commandInput);
    setResult(null);
    executeMutation.mutate({ command, args, riskLevel });
  };

  const getRiskMessage = () => {
    switch (riskLevel) {
      case 'LOW':
        return 'Command will be executed directly on the system.';
      case 'MEDIUM':
        return 'Command will be executed in a sandboxed environment (requires root).';
      case 'HIGH':
        return 'Command will be blocked for safety reasons.';
      default:
        return '';
    }
  };

  return (
    <Card title="Command Executor">
      <div className="space-y-4">
        {/* Command Input */}
        <div>
          <label 
            className="block text-sm font-medium mb-2"
            style={{ color: 'var(--ink-dark)' }}
          >
            Command
          </label>
          <div className="flex gap-2">
            <Terminal className="w-5 h-5 mt-2" style={{ color: 'var(--color-text-muted)' }} />
            <input
              type="text"
              value={commandInput}
              onChange={(e) => setCommandInput(e.target.value)}
              onKeyPress={(e) => e.key === 'Enter' && handleExecute()}
              placeholder="Enter command to execute..."
              className="flex-1 px-4 py-2 border rounded-lg focus:ring-2 focus:ring-[#622B14] focus:border-[#622B14] text-sm"
              style={{ borderColor: 'var(--color-border)', fontFamily: 'var(--font-mono)' }}
              disabled={executeMutation.isPending}
            />
          </div>
        </div>

        {/* Risk Level Selection */}
        <div>
          <label 
            className="block text-sm font-medium mb-2"
            style={{ color: 'var(--ink-dark)' }}
          >
            Risk Level
          </label>
          <div className="flex gap-2">
            {['LOW', 'MEDIUM', 'HIGH'].map((level) => {
              const isSelected = riskLevel === level;
              
              const buttonStyle = isSelected
                ? level === 'LOW'
                  ? {
                      backgroundColor: 'var(--color-success-bg)',
                      color: 'var(--color-success)',
                      borderColor: 'var(--color-success)',
                      borderWidth: '2px'
                    }
                  : level === 'MEDIUM'
                  ? {
                      backgroundColor: 'var(--color-warning-bg)',
                      color: 'var(--color-warning)',
                      borderColor: 'var(--color-warning)',
                      borderWidth: '2px'
                    }
                  : {
                      backgroundColor: 'var(--color-error-bg)',
                      color: 'var(--color-error)',
                      borderColor: 'var(--color-error)',
                      borderWidth: '2px'
                    }
                : {
                    backgroundColor: 'var(--color-bg-subtle)',
                    color: 'var(--color-text-muted)',
                    borderColor: 'transparent',
                    borderWidth: '2px'
                  };

              return (
                <button
                  key={level}
                  onClick={() => setRiskLevel(level)}
                  className="px-4 py-2 rounded-lg text-sm font-medium transition-colors"
                  style={buttonStyle}
                  disabled={executeMutation.isPending}
                >
                  {level}
                </button>
              );
            })}
          </div>
          <p className="text-xs mt-2" style={{ color: 'var(--color-text-muted)' }}>{getRiskMessage()}</p>
        </div>

        {/* Execute Button */}
        <button
          onClick={handleExecute}
          disabled={!commandInput.trim() || executeMutation.isPending}
          className="w-full flex items-center justify-center gap-2 px-4 py-3 text-white rounded-lg hover:opacity-90 disabled:opacity-50 disabled:cursor-not-allowed transition-colors font-medium"
          style={{ backgroundColor: 'var(--color-primary)' }}
        >
          {executeMutation.isPending ? (
            <>
              <LoadingSpinner size="sm" />
              <span>Executing...</span>
            </>
          ) : (
            <>
              <Play className="w-4 h-4" />
              <span>Execute Command</span>
            </>
          )}
        </button>

        {/* Result */}
        {result && (
          <div
            className="rounded-lg border p-4"
            style={
              result.success
                ? result.data.executed
                  ? { backgroundColor: 'var(--color-success-bg)', borderColor: 'var(--color-success)' }
                  : { backgroundColor: 'var(--color-warning-bg)', borderColor: 'var(--color-warning)' }
                : { backgroundColor: 'var(--color-error-bg)', borderColor: 'var(--color-error)' }
            }
          >
            <div className="flex items-start gap-3">
              {result.success ? (
                result.data.executed ? (
                  <CheckCircle className="w-5 h-5 flex-shrink-0 mt-0.5" style={{ color: 'var(--color-success)' }} />
                ) : (
                  <AlertTriangle className="w-5 h-5 flex-shrink-0 mt-0.5" style={{ color: 'var(--color-warning)' }} />
                )
              ) : (
                <XCircle className="w-5 h-5 flex-shrink-0 mt-0.5" style={{ color: 'var(--color-error)' }} />
              )}
              <div className="flex-1 space-y-2">
                <div className="flex items-center gap-2">
                  <span className="font-semibold text-sm">
                    {result.success
                      ? result.data.executed
                        ? 'Command Executed'
                        : 'Command Blocked'
                      : 'Execution Failed'}
                  </span>
                  {result.data?.method && (
                    <StatusBadge type="method">{result.data.method}</StatusBadge>
                  )}
                </div>
                <p className="text-sm">
                  {result.data?.message || result.error?.message || 'Unknown error'}
                </p>

                {result.data?.exit_code !== undefined && result.data?.exit_code !== null && (
                  <p className="text-sm">
                    <span className="font-medium">Exit Code:</span> {result.data.exit_code}
                  </p>
                )}

                {result.data?.duration_ms && (
                  <p className="text-sm">
                    <span className="font-medium">Duration:</span> {formatDuration(result.data.duration_ms)}
                  </p>
                )}

                {result.data?.stdout && (
                  <div className="mt-2">
                    <label className="text-xs font-semibold mb-1 block">Output:</label>
                    <pre 
                      className="text-xs border rounded p-2 overflow-x-auto max-h-40"
                      style={{ 
                        backgroundColor: 'var(--offwhite-main)', 
                        borderColor: 'var(--color-border)',
                        fontFamily: 'var(--font-mono)' 
                      }}
                    >
                      {result.data.stdout}
                    </pre>
                  </div>
                )}

                {result.data?.stderr && (
                  <div className="mt-2">
                    <label className="text-xs font-semibold mb-1 block" style={{ color: 'var(--color-error)' }}>Error:</label>
                    <pre 
                      className="text-xs border rounded p-2 overflow-x-auto max-h-40"
                      style={{ 
                        backgroundColor: 'var(--color-error-bg)', 
                        borderColor: 'var(--color-error)',
                        fontFamily: 'var(--font-mono)' 
                      }}
                    >
                      {result.data.stderr}
                    </pre>
                  </div>
                )}
              </div>
            </div>
          </div>
        )}
      </div>
    </Card>
  );
}
