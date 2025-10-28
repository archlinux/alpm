/**
 * A helper class to filter the list of all lints based on search criteria.
 *
 * The search criteria are extracted from the following elements:
 * - The text search input field
 * - Dropdown selectors for
 *   - Groups
 *   - Scopes
 *   - Levels
 */
class LintFilter {
    constructor() {
        this.searchInput = document.getElementById("search-input");
        this.clearButton = document.getElementById("clear-filters");
        this.resultsContainer = document.getElementById("lint-rules-container");
        this.noResultsDiv = document.getElementById("no-results");
        this.resultsCounter = document.getElementById("visible-count");
        this.dropdowns = document.querySelectorAll(".dropdown");

        this.filters = {
            search: "",
            groups: [],
            scopes: [],
            levels: [],
        };

        this.init();
    }

    init() {
        this.setupDropdowns();
        this.setupEventListeners();
    }

    setupDropdowns() {
        // First up, clear all selects in case the browser saved some from the last session.
        this.clearAllFilters();

        // Go through all dropdown fields and set them up.
        //
        // Create a toggle toggle on the inputs that open up the dropdown fields.
        // Also adds a global clicking handler that closes all dropdowns if clicked anywhere else.
        for (const dropdown of this.dropdowns) {
            const checkbox_container = dropdown.querySelector(".checkbox-container");
            const toggle = dropdown.querySelector(".dropdown-toggle");
            const _checkboxes = checkbox_container.querySelectorAll(
                'input[type="checkbox"]',
            );

            // Toggle dropdown
            toggle.addEventListener("click", (e) => {
                e.stopPropagation();

                // Only close other dropdowns if we're **opening** this dropdown
                const isCurrentlyOpen =
                    !checkbox_container.classList.contains("hidden");
                if (!isCurrentlyOpen) {
                    this.closeAllDropdowns();
                }

                checkbox_container.classList.toggle("hidden");
                toggle.classList.toggle("active");
            });

            // Prevent dropdown interior clicks from bubbling up
            //
            // This otherwise toggle the global document click handler we set up
            // below to close dropdowns when clicking anywhere in the document.
            checkbox_container.addEventListener("click", (e) => {
                e.stopPropagation();
            });
        }

        // Close dropdowns when clicking outside
        document.addEventListener("click", (e) => {
            // Only close if click is outside all dropdowns
            const isInsideDropdown = e.target.closest(".dropdown");
            if (!isInsideDropdown) {
                this.closeAllDropdowns();
            }
        });
    }

    // Register event listeners for all
    setupEventListeners() {
        // Search input
        this.searchInput.addEventListener("input", (e) => {
            this.filters.search = e.target.value.toLowerCase();
            this.applyFilters();
        });

        // Setup checkbox change handlers for checkboxes.
        for (const dropdown of this.dropdowns) {
            const checkbox_container = dropdown.querySelector(".checkbox-container");
            const toggle = dropdown.querySelector(".dropdown-toggle");
            const checkboxes = checkbox_container.querySelectorAll(
                'input[type="checkbox"]',
            );
            const selectedText = toggle.querySelector(".selected-text");
            // Handle checkbox changes
            for (const checkbox of checkboxes) {
                checkbox.addEventListener("change", () => {
                    this.handleCheckboxChange(dropdown, selectedText);
                    this.applyFilters();
                });
            }
        }

        // Clear filters button
        this.clearButton.addEventListener("click", () => {
            this.clearAllFilters();
        });
    }

    handleCheckboxChange(selectElement, selectedTextElement) {
        const filterType = selectElement.dataset.filter;
        const checkboxes = selectElement.querySelectorAll('input[type="checkbox"]');

        const selectedValues = [];
        const selectedLabels = [];

        for (const checkbox of checkboxes) {
            if (checkbox.checked) {
                selectedValues.push(checkbox.value);
                selectedLabels.push(checkbox.dataset.label);
            }
        }

        // Update display text and filter state
        if (selectedValues.length === 0) {
            selectedTextElement.textContent = `All ${filterType.charAt(0).toUpperCase() + filterType.slice(1)}`;
            this.filters[filterType] = [];
        } else if (selectedLabels.length === 1) {
            selectedTextElement.textContent = selectedLabels[0];
            this.filters[filterType] = selectedValues;
        } else {
            selectedTextElement.textContent = `${selectedLabels.length} selected`;
            this.filters[filterType] = selectedValues;
        }
    }

    closeAllDropdowns() {
        for (const dropdown of this.dropdowns) {
            const checkbox_container = dropdown.querySelector(".checkbox-container");
            const toggle = dropdown.querySelector(".dropdown-toggle");
            checkbox_container.classList.add("hidden");
            toggle.classList.remove("active");
        }
    }

    clearAllFilters() {
        // Clear search
        this.searchInput.value = "";
        this.filters.search = "";

        // Reset all dropdowns
        for (const dropdown of this.dropdowns) {
            // Uncheck all checkboxes
            const checkboxes = dropdown.querySelectorAll('input[type="checkbox"]');
            for (const checkbox of checkboxes) {
                checkbox.checked = false;
            }

            // Clear filter
            const filterType = dropdown.dataset.filter;
            this.filters[filterType] = [];
        }

        this.applyFilters();
    }

    applyFilters() {
        const lintRules = document.querySelectorAll(".lint-rule");
        let visibleCount = 0;

        // Go through each rule and check whether it should be shown/hidden.
        for (const rule of lintRules) {
            let isVisible = true;

            // Perform a simple substring check if there's something in the search field.
            if (this.filters.search) {
                const searchableText = rule.dataset.scopedName;

                isVisible = isVisible && searchableText.includes(this.filters.search);
            }

            // By default, just show all rules as if all groups were enabled.
            // If at least one group is selected, only show the rules that have all selected groups.
            if (this.filters.groups.length > 0) {
                const ruleGroups = rule.dataset.groups.split(",").filter((g) => g);
                console.log(ruleGroups);

                // The `none` group is a special case, where only rules without any groups are shown.
                if (this.filters.groups.includes("none")) {
                    // Include rules with no groups if "none" is selected
                    isVisible = isVisible && ruleGroups.length === 0;
                } else {
                    // Only include rules where all groups match
                    isVisible =
                        isVisible &&
                        ruleGroups.length !== 0 &&
                        this.filters.groups.every((filter) => ruleGroups.includes(filter));
                }
            }

            // Filter out all rules whose scope isn't selected.
            // Shows rules of all scopes by default.
            if (this.filters.scopes.length > 0) {
                isVisible =
                    isVisible && this.filters.scopes.includes(rule.dataset.scope);
            }

            // Filter out all rules whose level isn't selected.
            // Shows rules of all levels by default.
            if (this.filters.levels.length > 0) {
                isVisible =
                    isVisible && this.filters.levels.includes(rule.dataset.level);
            }

            // Only show the rule if all conditions matched.
            if (isVisible) {
                rule.classList.remove("hidden");
                visibleCount++;
            } else {
                rule.classList.add("hidden");
            }
        }

        // Update results counter
        this.resultsCounter.textContent = visibleCount;

        // Show/hide no results message
        if (visibleCount === 0) {
            this.noResultsDiv.classList.remove("hidden");
            this.resultsContainer.classList.add("hidden");
        } else {
            this.noResultsDiv.classList.add("hidden");
            this.resultsContainer.classList.remove("hidden");
        }
    }
}

// Initialize when DOM is loaded
document.addEventListener("DOMContentLoaded", () => {
    new LintFilter();
});
