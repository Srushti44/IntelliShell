/**
 * IntelliShell Dashboard - Entry Point (Phase 7)
 */
import { StrictMode } from 'react';
import { createRoot } from 'react-dom/client';
import './styles/colors.css';
import './styles/typography.css';
import './index.css';
import App from './App.jsx';

createRoot(document.getElementById('root')).render(
  <StrictMode>
    <App />
  </StrictMode>,
);
