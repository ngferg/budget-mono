<template>
    <div class="landing">
        <!-- Hero -->
        <section class="hero">
            <h2>Your Budget. Your Data. <span class="accent">No One Else's.</span></h2>
            <p class="hero-subtitle">
                FeBudget is a private, open-source budgeting tool built from the ground up to keep your financial data
                truly yours. No shared databases. Just a clean, fast way to manage your money.
            </p>
        </section>

        <!-- Feature Grid -->
        <section class="features">
            <div class="feature-card">
                <div class="feature-icon">🔒</div>
                <h3>Isolated, Anonymized Storage</h3>
                <p>
                    Every user's data lives in its own individual database file, named with a one-way hash — not your
                    email, not your name. Even if someone gained access to the server, they'd see nothing but anonymous
                    files with no way to tie them back to you.
                </p>
            </div>

            <div class="feature-card">
                <div class="feature-icon">🚫</div>
                <h3>No Passwords to Steal</h3>
                <p>
                    FeBudget uses passwordless login. Enter your email, receive a one-time code, and you're in. No
                    password database means no password breaches — ever.
                </p>
            </div>

            <div class="feature-card">
                <div class="feature-icon">🧹</div>
                <h3>True Data Deletion</h3>
                <p>
                    Want to leave? Deleting your account wipes your entire isolated database in one step. No orphaned
                    rows, no lingering records, no "we'll delete it in 30 days." It's gone — immediately.
                </p>
            </div>

            <div class="feature-card">
                <div class="feature-icon">⚡</div>
                <h3>Fast &amp; Lightweight</h3>
                <p>
                    Built with Rust on the backend and Vue on the frontend — no bloated frameworks, no slow queries
                    against massive shared tables. Your own small database means lightning-fast responses.
                </p>
            </div>

            <div class="feature-card">
                <div class="feature-icon">📋</div>
                <h3>Monthly Budgets Made Simple</h3>
                <p>
                    Organize income and expenses by category, add line items, and see your totals at a glance. Starting
                    a new month? Clone last month's budget with one click so you never start from scratch.
                </p>
            </div>

            <div class="feature-card">
                <div class="feature-icon">🌐</div>
                <h3>Fully Open Source</h3>
                <p>
                    Every line of code is public. Audit the privacy model yourself, self-host it, or contribute.
                    No black boxes, no hidden behavior — just transparent software you can trust.
                    <a href="https://github.com/ngferg/budget-mono" target="_blank" class="source-link">View on GitHub
                        →</a>
                </p>
            </div>

        </section>

        <!-- How It Works -->
        <section class="how-it-works">
            <h3>How It Works</h3>
            <div class="steps">
                <div class="step">
                    <div class="step-number">1</div>
                    <p><strong>Enter your email</strong> — we send you a one-time login code.</p>
                </div>
                <div class="step">
                    <div class="step-number">2</div>
                    <p><strong>Verify the code</strong> — no password needed, ever.</p>
                </div>
                <div class="step">
                    <div class="step-number">3</div>
                    <p><strong>Start budgeting</strong> — your data is stored in your own private, anonymized database.
                    </p>
                </div>
            </div>
        </section>

        <!-- Privacy Callout -->
        <section class="privacy-callout">
            <h3>Privacy by Architecture, Not by Promise</h3>
            <p>
                Most apps promise privacy in a terms-of-service document. FeBudget enforces it structurally.
                Your financial data is stored in a standalone database identified only by an irreversible hash.
                There is no central user table. Your email is never written to disk. The system literally
                <em>cannot</em> associate your data with your identity at rest — because the architecture makes it
                impossible, not because a policy says so.
            </p>
        </section>
    </div>
</template>

<style scoped>
.landing {
    max-width: 860px;
    margin: 0 auto;
    padding: 0 1.5rem 4rem;
}

/* Hero */
.hero {
    text-align: center;
    padding: 3rem 0 2.5rem;
}

.hero h2 {
    font-size: 2.4rem;
    font-weight: 800;
    color: #d1fae5;
    line-height: 1.2;
    margin-bottom: 1rem;
}

.accent {
    color: #34d399;
}

.hero-subtitle {
    color: #a7f3d0;
    font-size: 1.15rem;
    line-height: 1.7;
    max-width: 620px;
    margin: 0 auto;
}

/* Feature Grid */
.features {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(250px, 1fr));
    gap: 1.25rem;
    margin-top: 1rem;
}

.feature-card {
    background: linear-gradient(135deg, #0d2016 0%, #012305 100%);
    border: 1px solid rgba(16, 185, 129, 0.25);
    border-radius: 12px;
    padding: 1.5rem;
    transition: border-color 0.2s ease, transform 0.2s ease;
}

.feature-card:hover {
    border-color: #34d399;
    transform: translateY(-2px);
}

.feature-icon {
    font-size: 1.8rem;
    margin-bottom: 0.6rem;
}

.feature-card h3 {
    color: #34d399;
    font-size: 1.1rem;
    font-weight: 700;
    margin: 0 0 0.5rem;
}

.feature-card p {
    color: #a7f3d0;
    font-size: 0.92rem;
    line-height: 1.6;
    margin: 0;
}

.source-link {
    display: inline-block;
    margin-top: 0.6rem;
    color: #34d399;
    font-weight: 600;
    text-decoration: none;
    transition: color 0.2s ease;
}

.source-link:hover {
    color: #6ee7b7;
    text-decoration: underline;
}

/* How It Works */
.how-it-works {
    margin-top: 3rem;
    text-align: center;
}

.how-it-works h3 {
    color: #d1fae5;
    font-size: 1.6rem;
    font-weight: 700;
    margin-bottom: 1.5rem;
}

.steps {
    display: flex;
    justify-content: center;
    gap: 2rem;
    flex-wrap: wrap;
}

.step {
    display: flex;
    align-items: flex-start;
    gap: 0.75rem;
    max-width: 240px;
    text-align: left;
}

.step-number {
    flex-shrink: 0;
    width: 2rem;
    height: 2rem;
    border-radius: 50%;
    background: #10b981;
    color: #022c22;
    font-weight: 800;
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: 0.95rem;
}

.step p {
    color: #a7f3d0;
    font-size: 0.92rem;
    line-height: 1.5;
    margin: 0;
}

.step strong {
    color: #d1fae5;
}

/* Privacy Callout */
.privacy-callout {
    margin-top: 3rem;
    background: linear-gradient(135deg, #0d2016 0%, #012305 100%);
    border: 1px solid rgba(16, 185, 129, 0.3);
    border-radius: 12px;
    padding: 2rem;
    text-align: center;
    box-shadow: 0 10px 30px rgba(0, 0, 0, 0.3), inset 0 0 1px rgba(16, 185, 129, 0.1);
}

.privacy-callout h3 {
    color: #34d399;
    font-size: 1.35rem;
    font-weight: 700;
    margin: 0 0 0.75rem;
}

.privacy-callout p {
    color: #a7f3d0;
    font-size: 1rem;
    line-height: 1.7;
    max-width: 700px;
    margin: 0 auto;
}

.privacy-callout em {
    color: #d1fae5;
    font-style: italic;
}
</style>
