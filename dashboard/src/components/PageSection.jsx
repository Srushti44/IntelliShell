/**
 * Page Section Component
 * Swiss-style content sections
 */

export function PageSection({ title, description, children, className }) {
  return (
    <section className={`mb-12 ${className || ''}`}>
      {(title || description) && (
        <div className="mb-6">
          {title && (
            <h3 
              className="font-heading font-semibold text-xl tracking-tight mb-2"
              style={{ color: 'var(--ink-dark)' }}
            >
              {title}
            </h3>
          )}
          {description && (
            <p 
              className="text-base max-w-3xl"
              style={{ color: 'var(--color-text-muted)' }}
            >
              {description}
            </p>
          )}
        </div>
      )}
      {children}
    </section>
  );
}


export default PageSection;
