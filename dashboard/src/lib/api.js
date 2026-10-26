/**
 * API Client for IntelliShell Backend Services
 * Handles communication with both Rust (port 3000) and Python (port 8000) servers
 */

import axios from 'axios';

const RUST_API_URL = import.meta.env.VITE_RUST_API_URL || 'http://localhost:3000';
const PYTHON_API_URL = import.meta.env.VITE_PYTHON_API_URL || 'http://localhost:8000';

// Create axios instances with default config
const rustClient = axios.create({
  baseURL: RUST_API_URL,
  timeout: 10000,
  headers: {
    'Content-Type': 'application/json',
  },
});

const pythonClient = axios.create({
  baseURL: PYTHON_API_URL,
  timeout: 60000, // Longer timeout for AI operations
  headers: {
    'Content-Type': 'application/json',
  },
});

// Error handler
const handleError = (error, serviceName) => {
  if (error.response) {
    // Server responded with error status
    throw {
      type: 'SERVER_ERROR',
      status: error.response.status,
      message: error.response.data?.detail || error.response.data?.message || 'Server error',
      service: serviceName,
    };
  } else if (error.request) {
    // Request made but no response
    throw {
      type: 'NO_RESPONSE',
      message: `${serviceName} is not responding. Is the server running?`,
      service: serviceName,
    };
  } else {
    // Something else went wrong
    throw {
      type: 'CLIENT_ERROR',
      message: error.message || 'Unknown error occurred',
      service: serviceName,
    };
  }
};

/**
 * Rust Server API (Core Engine)
 */
export const rustApi = {
  /**
   * Health check
   */
  async health() {
    try {
      const response = await rustClient.get('/health');
      return response.data;
    } catch (error) {
      handleError(error, 'Rust Server');
    }
  },

  /**
   * Get system snapshot (CPU, RAM, processes)
   */
  async getSnapshot() {
    try {
      const response = await rustClient.get('/snapshot');
      return response.data;
    } catch (error) {
      handleError(error, 'Rust Server');
    }
  },

  /**
   * Execute command
   * @param {string} command - Command to execute
   * @param {string[]} args - Command arguments
   * @param {string} riskLevel - LOW | MEDIUM | HIGH
   */
  async executeCommand(command, args, riskLevel) {
    try {
      const response = await rustClient.post('/execute', {
        command,
        args,
        risk_level: riskLevel,
      });
      return response.data;
    } catch (error) {
      handleError(error, 'Rust Server');
    }
  },

  /**
   * Get command history
   */
  async getHistory() {
    try {
      const response = await rustClient.get('/history');
      return response.data;
    } catch (error) {
      handleError(error, 'Rust Server');
    }
  },

  /**
   * List all processes
   */
  async listProcesses() {
    try {
      const response = await rustClient.get('/processes');
      return response.data;
    } catch (error) {
      handleError(error, 'Rust Server');
    }
  },

  /**
   * Control a process
   * @param {number} pid - Process ID
   * @param {string} action - terminate | suspend | resume
   */
  async controlProcess(pid, action) {
    try {
      const response = await rustClient.post(`/processes/${pid}`, { action });
      return response.data;
    } catch (error) {
      handleError(error, 'Rust Server');
    }
  },
};

/**
 * Python AI Service API
 */
export const pythonApi = {
  /**
   * Health check
   */
  async health() {
    try {
      const response = await pythonClient.get('/health');
      return response.data;
    } catch (error) {
      handleError(error, 'Python AI Service');
    }
  },

  /**
   * Generate command from natural language
   * @param {string} query - Natural language description
   */
  async generateCommand(query) {
    try {
      const response = await pythonClient.post('/generate-command', { query });
      return response.data;
    } catch (error) {
      handleError(error, 'Python AI Service');
    }
  },

  /**
   * Confirm and execute command with token
   * @param {string} token - Confirmation token from generate-command
   */
  async confirmExecute(token) {
    try {
      const response = await pythonClient.post('/confirm-execute', { token });
      return response.data;
    } catch (error) {
      handleError(error, 'Python AI Service');
    }
  },

  /**
   * Troubleshoot system issue
   * @param {string} issueDescription - Description of the problem
   */
  async troubleshoot(issueDescription) {
    try {
      const response = await pythonClient.post('/troubleshoot', {
        issue_description: issueDescription,
      });
      return response.data;
    } catch (error) {
      handleError(error, 'Python AI Service');
    }
  },

  /**
   * Get activity audit log
   */
  async getActivity() {
    try {
      const response = await pythonClient.get('/activity');
      return response.data;
    } catch (error) {
      handleError(error, 'Python AI Service');
    }
  },

  /**
   * Get command history (proxies to Rust)
   */
  async getHistory() {
    try {
      const response = await pythonClient.get('/history');
      return response.data;
    } catch (error) {
      handleError(error, 'Python AI Service');
    }
  },

  /**
   * Classify command risk level (legacy)
   * @param {string} command - Command to classify
   */
  async classify(command) {
    try {
      const response = await pythonClient.post('/classify', { command });
      return response.data;
    } catch (error) {
      handleError(error, 'Python AI Service');
    }
  },

  /**
   * Get safer alternative (legacy)
   * @param {string} command - Command to analyze
   */
  async suggest(command) {
    try {
      const response = await pythonClient.post('/suggest', { command });
      return response.data;
    } catch (error) {
      handleError(error, 'Python AI Service');
    }
  },
};

/**
 * Combined health check for both services
 */
export async function checkAllServices() {
  try {
    const [rustHealth, pythonHealth] = await Promise.allSettled([
      rustApi.health(),
      pythonApi.health(),
    ]);

    return {
      rust: {
        status: rustHealth.status === 'fulfilled' ? 'online' : 'offline',
        data: rustHealth.status === 'fulfilled' ? rustHealth.value : null,
        error: rustHealth.status === 'rejected' ? rustHealth.reason : null,
      },
      python: {
        status: pythonHealth.status === 'fulfilled' ? 'online' : 'offline',
        data: pythonHealth.status === 'fulfilled' ? pythonHealth.value : null,
        error: pythonHealth.status === 'rejected' ? pythonHealth.reason : null,
      },
    };
  } catch (error) {
    throw {
      type: 'HEALTH_CHECK_FAILED',
      message: 'Failed to check service health',
    };
  }
}
