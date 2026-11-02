/**
 * AboutPage - IntelliShell Project & OS Concepts Showcase
 * Frontend-only enhancement showcasing features and OS theory implementation
 */

import React from 'react';
import { ProjectOverview } from '../components/ProjectOverview';
import OSConceptsShowcase from '../components/OSConceptsShowcase';
import '../styles/AboutPage.css';

export function AboutPage() {
  return (
    <div className="about-page">
      <div className="about-container">
        {/* Project Overview Section */}
        <ProjectOverview />

        {/* OS Concepts Section */}
        <OSConceptsShowcase />
      </div>
    </div>
  );
}

export default AboutPage;
