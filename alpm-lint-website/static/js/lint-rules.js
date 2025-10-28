/**
 * A helper class that handles lint rule collapsing.
 *
 * Specifically it
 * - sets the anchor to the currently selected lint rule
 * - handles deeplings for specific lint rules on page load
 * - handles the toggle/expand logic for lint rules
 */
class LintRules {
    constructor() {
        this.expandedRules = new Set();
        this.init();
    }

    init() {
        this.setupCollapsibleRules();
        this.handleDeepLink();
        this.setupScopedNameBreaking();
    }

    // Set the on-click handlers for all lint rule elements.
    setupCollapsibleRules() {
        const ruleHeaders = document.querySelectorAll(".lint-rule-header");

        for (const header of ruleHeaders) {
            header.addEventListener("click", (e) => {
                e.preventDefault();
                const ruleElement = header.closest(".lint-rule");
                const ruleId = ruleElement.id;
                this.toggleRule(ruleId);
            });
        }
    }

    toggleRule(ruleId) {
        const ruleElement = document.getElementById(ruleId);
        if (!ruleElement) return;

        const isExpanded = this.expandedRules.has(ruleId);

        if (isExpanded) {
            this.collapseRule(ruleId);
        } else {
            this.expandRule(ruleId);
        }
    }

    expandRule(ruleId) {
        const ruleElement = document.getElementById(ruleId);
        if (!ruleElement) return;

        const content = ruleElement.querySelector(".lint-rule-content");

        // Add expanded state
        content.classList.remove("collapsed");
        content.classList.add("expanded");

        // Track state
        this.expandedRules.add(ruleId);

        // Update URL
        this.updateURL(ruleId);
    }

    collapseRule(ruleId) {
        const ruleElement = document.getElementById(ruleId);
        if (!ruleElement) return;

        const content = ruleElement.querySelector(".lint-rule-content");

        // Remove expanded state
        content.classList.remove("expanded");
        content.classList.add("collapsed");

        // Track state
        this.expandedRules.delete(ruleId);

        // Clear URL if this was the active rule
        if (window.location.hash === `#${ruleId}`) {
            this.clearURL();
        }
    }

    // Update URL hash without triggering scroll
    updateURL(ruleId) {
        const newURL = `${window.location.pathname + window.location.search}#${ruleId}`;
        history.replaceState(null, null, newURL);
    }

    // Clear the URL hash
    clearURL() {
        const newURL = window.location.pathname + window.location.search;
        history.replaceState(null, null, newURL);
    }

    // Automatically open and focus any rule that's referenced via a `#scoped::rule-name` tag.
    handleDeepLink() {
        const hash = window.location.hash;
        if (hash && hash.length > 1) {
            const ruleId = hash.substring(1); // Remove the # character
            const ruleElement = document.getElementById(ruleId);

            // Don't do anything if the rule doesn't exist.
            if (ruleElement) {
                this.expandRule(ruleId);

                // Scroll to the rule after a short delay to ensure the expansion animation has finished.
                // Otherwise the position is just a bit off.
                setTimeout(() => {
                    ruleElement.scrollIntoView({
                        behavior: "smooth",
                        block: "start",
                    });
                }, 100);
            }
        }
    }

    // Collapse all rules
    collapseAllRules() {
        const expandedRuleIds = Array.from(this.expandedRules);
        for (const ruleId of expandedRuleIds) {
            this.collapseRule(ruleId);
        }
        this.clearURL();
    }

    /* Add word-break opportunities for scoped names.
     * This is done by adding a zero-width space (&#8203;) after `::` to allow
     * **satisfactory** breaking for small screens.
     *
     * Without this, text breaks char-by-char for small screens, which is just ugly.
     */
    setupScopedNameBreaking() {
        const scopedNames = document.querySelectorAll(".scoped-name");

        for (const nameElement of scopedNames) {
            const text = nameElement.textContent;
            const breakableText = text.replace(/::/g, "::&#8203;");
            nameElement.innerHTML = breakableText;
        }
    }
}

// Initialize when DOM is loaded
document.addEventListener("DOMContentLoaded", () => {
    window.lintRules = new LintRules();
});
