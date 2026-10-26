/**
 * AI Command Assistant Page
 * Natural language to command generation
 */

import { useState } from 'react';
import { useMutation } from '@tanstack/react-query';
import { pythonApi } from '../lib/api';
import { TopBar } from '../components/TopBar';
import { PageSection } from '../components/PageSection';
import { Sparkles, AlertTriangle, CheckCircle, Copy } from 'lucide-react';
import { StatusBadge } from '../components/StatusBadge';
import { LoadingSpinner } from '../components/LoadingSpinner';
import { ErrorAlert } from '../components/ErrorAlert';

export function AssistantPage() {
  const [query, setQuery] = useState('');
  const [generated, setGenerated] = useState(null);
  const [executionResult, setExecutionResult] = useState(null);

  const generateMutation = useMutation({
    mutationFn: (query) => pythonApi.generateCommand(query),
    onSuccess: (data) => {
      setGenerated(data);
      setExecutionResult(null); // Clear previous execution result
    },
  });

  const confirmExecuteMutation = useMutation({
    mutationFn: (token) => pythonApi.confirmExecute(token),
    onSuccess: (data) => {
      setExecutionResult(data);
    },
  });

  const handleGenerate = () => {
    if (!query.trim()) return;
    setGenerated(null);
    setExecutionResult(null);
    generateMutation.mutate(query);
  };

  const handleCopy = (text) => {
    navigator.clipboard.writeText(text);
  };

  const handleExecute = () => {
    if (!generated?.confirmation_token) return;
    setExecutionResult(null);
    confirmExecuteMutation.mutate(generated.confirmation_token);
  };

  return (
    <>
      <TopBar 
        title="AI Command Assistant" 
        subtitle="Generate Linux commands from natural language"
      />
      
      <main className="p-8" style={{ backgroundColor: 'var(--color-bg-subtle)' }}>
        <PageSection
          description="Describe what you want to accomplish in plain language. IntelliShell will generate the appropriate Linux command, explain what it does, assess its risk level, and suggest safer alternatives when applicable. Review all generated commands carefully before execution."
        >
          {/* Input Form */}
          <div className="bg-white rounded-lg border p-6 mb-6" style={{ borderColor: 'var(--color-border)' }}>
            <label className="block text-sm font-medium mb-3" style={{ color: 'var(--ink-dark)' }}>
              What would you like to do?
            </label>
            <textarea
              value={query}
              onChange={(e) => setQuery(e.target.value)}
              onKeyDown={(e) => {
                if (e.key === 'Enter' && e.ctrlKey) {
                  handleGenerate();
                }
              }}
              placeholder="Example: List all files modified in the last 24 hours..."
              rows={4}
              className="w-full px-4 py-3 border rounded-md font-body text-base resize-none"
              style={{
                borderColor: 'var(--color-border)',
                color: 'var(--ink-dark)',
              }}
              disabled={generateMutation.isPending}
            />
            <div className="flex items-center justify-between mt-4">
              <p className="text-xs" style={{ color: 'var(--color-text-muted)' }}>
                Press Ctrl+Enter to generate
              </p>
              <button
                onClick={handleGenerate}
                disabled={!query.trim() || generateMutation.isPending}
                className="flex items-center gap-2 px-6 py-2.5 rounded-md font-medium transition-colors disabled:opacity-50"
                style={{
                  backgroundColor: 'var(--brown-deep)',
                  color: 'var(--cream-warm)',
                }}
              >
                {generateMutation.isPending ? (
                  <>
                    <LoadingSpinner size="sm" />
                    <span>Generating...</span>
                  </>
                ) : (
                  <>
                    <Sparkles className="w-4 h-4" />
                    <span>Generate Command</span>
                  </>
                )}
              </button>
            </div>
          </div>

          {/* Error State */}
          {generateMutation.isError && (
            <ErrorAlert
              title="Generation Failed"
              message={generateMutation.error?.message || 'Could not generate command'}
              onRetry={() => generateMutation.mutate(query)}
            />
          )}

          {/* Generated Result */}
          {generated && (
            <div className="bg-white rounded-lg border p-6" style={{ borderColor: 'var(--color-border)' }}>
              <div className="flex items-center justify-between mb-4">
                <h4 className="font-heading font-semibold text-lg" style={{ color: 'var(--ink-dark)' }}>
                  Generated Command
                </h4>
                <StatusBadge type="risk">{generated.risk_level}</StatusBadge>
              </div>

              {/* Command */}
              <div className="bg-subtle rounded-md p-4 mb-4 flex items-center justify-between" style={{ backgroundColor: 'var(--color-bg-subtle)' }}>
                <code className="font-mono text-base" style={{ color: 'var(--ink-dark)' }}>
                  {generated.command}
                </code>
                <button
                  onClick={() => handleCopy(generated.command)}
                  className="flex-shrink-0 ml-4 p-2 rounded hover:bg-opacity-10 hover:bg-black"
                  style={{ color: 'var(--color-text-muted)' }}
                  title="Copy command"
                >
                  <Copy className="w-4 h-4" />
                </button>
              </div>

              {/* Explanation */}
              <div className="mb-4">
                <p className="text-sm font-medium mb-2" style={{ color: 'var(--color-text-muted)' }}>
                  What this does:
                </p>
                <p className="text-base" style={{ color: 'var(--ink-dark)' }}>
                  {generated.explanation}
                </p>
              </div>

              {/* Risk Reason */}
              {generated.risk_reason && (
                <div className="mb-4 p-3 rounded-md" style={{ backgroundColor: 'var(--color-info-bg)' }}>
                  <p className="text-sm flex items-start gap-2" style={{ color: 'var(--color-info)' }}>
                    <AlertTriangle className="w-4 h-4 flex-shrink-0 mt-0.5" />
                    <span>{generated.risk_reason}</span>
                  </p>
                </div>
              )}

              {/* Safer Alternative */}
              {generated.safer_alternative && (
                <div className="border-t pt-4" style={{ borderColor: 'var(--color-border)' }}>
                  <p className="text-sm font-medium mb-2 flex items-center gap-2" style={{ color: 'var(--color-success)' }}>
                    <CheckCircle className="w-4 h-4" />
                    Safer alternative:
                  </p>
                  <div className="bg-subtle rounded-md p-3" style={{ backgroundColor: 'var(--color-success-bg)' }}>
                    <code className="font-mono text-sm" style={{ color: 'var(--color-success)' }}>
                      {generated.safer_alternative}
                    </code>
                  </div>
                </div>
              )}

              {/* Token Info */}
              {generated.confirmation_token && !executionResult && (
                <div className="mt-4 p-3 rounded-md" style={{ backgroundColor: 'var(--color-warning-bg)' }}>
                  <p className="text-xs" style={{ color: 'var(--color-warning)' }}>
                    This command requires confirmation. Use the confirmation token within the time limit to execute.
                  </p>
                </div>
              )}

              {/* Execute Button */}
              {generated.confirmation_token && !executionResult && (
                <div className="mt-4">
                  <button
                    onClick={handleExecute}
                    disabled={confirmExecuteMutation.isPending}
                    className="flex items-center gap-2 px-6 py-2.5 rounded-md font-medium transition-colors disabled:opacity-50"
                    style={{
                      backgroundColor: 'var(--brown-deep)',
                      color: 'var(--cream-warm)',
                    }}
                  >
                    {confirmExecuteMutation.isPending ? (
                      <>
                        <LoadingSpinner size="sm" />
                        <span>Executing...</span>
                      </>
                    ) : (
                      <>
                        <CheckCircle className="w-4 h-4" />
                        <span>Execute Command</span>
                      </>
                    )}
                  </button>
                </div>
              )}

              {/* Execution Error */}
              {confirmExecuteMutation.isError && (
                <div className="mt-4">
                  <ErrorAlert
                    title="Execution Failed"
                    message={confirmExecuteMutation.error?.message || 'Could not execute command'}
                    onRetry={() => confirmExecuteMutation.mutate(generated.confirmation_token)}
                  />
                </div>
              )}

              {/* Execution Result */}
              {executionResult && (
                <div className="mt-4 border rounded-lg p-4" style={{ borderColor: 'var(--color-border)' }}>
                  <div className="flex items-center justify-between mb-3">
                    <h5 className="font-heading font-semibold" style={{ color: 'var(--ink-dark)' }}>
                      Execution Result
                    </h5>
                    <StatusBadge type={executionResult.executed ? 'success' : 'error'}>
                      {executionResult.executed ? executionResult.method : 'blocked'}
                    </StatusBadge>
                  </div>
                  
                  {executionResult.message && (
                    <p className="text-sm mb-3" style={{ color: 'var(--color-text-muted)' }}>
                      {executionResult.message}
                    </p>
                  )}
                  
                  {executionResult.exit_code !== null && executionResult.exit_code !== undefined && (
                    <p className="text-sm mb-2">
                      <strong>Exit Code:</strong> {executionResult.exit_code}
                    </p>
                  )}
                  
                  {executionResult.stdout && (
                    <div className="mb-3">
                      <p className="text-sm font-medium mb-1">Output:</p>
                      <pre className="bg-subtle p-3 rounded text-sm overflow-x-auto" style={{ backgroundColor: 'var(--color-bg-subtle)' }}>
                        {executionResult.stdout}
                      </pre>
                    </div>
                  )}
                  
                  {executionResult.stderr && (
                    <div>
                      <p className="text-sm font-medium mb-1" style={{ color: 'var(--color-error)' }}>
                        Error Output:
                      </p>
                      <pre className="bg-subtle p-3 rounded text-sm overflow-x-auto" style={{ backgroundColor: 'var(--color-error-bg)', color: 'var(--color-error)' }}>
                        {executionResult.stderr}
                      </pre>
                    </div>
                  )}
                  
                  {executionResult.duration_ms && (
                    <p className="text-xs mt-3" style={{ color: 'var(--color-text-muted)' }}>
                      Execution time: {executionResult.duration_ms}ms
                    </p>
                  )}
                </div>
              )}
            </div>
          )}

          {/* Empty State */}
          {!generated && !generateMutation.isPending && !generateMutation.isError && (
            <div className="text-center py-12" style={{ color: 'var(--color-text-muted)' }}>
              <Sparkles className="w-12 h-12 mx-auto mb-3 opacity-50" />
              <p>Enter a description above to generate a command</p>
            </div>
          )}
        </PageSection>
      </main>
    </>
  );
}


export default AssistantPage;
