(() => {
    const addKeydownToEach = (selector, action) => {
        document
            .querySelectorAll(selector)
            .forEach(element => element.addEventListener("keydown", action));
    };

    // Make <enter> and <space> toggle hidden-checkbox-based collapsibles
    addKeydownToEach(".collapse--lbl-toggle", e => {
        if (e.key === "Enter" || e.key === " ") {
            e.preventDefault();
            e.target.click();
        }
    });

    // Make forms submit on ctrl+enter in textareas
    addKeydownToEach("textarea", e => {
        if (e.key === "Enter" && e.ctrlKey) {
            e.target.form.submit();
        }
    });
})();
