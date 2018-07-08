(() => {
    // Make <enter> and <space> toggle hidden-checkbox-based collapsables
    document.querySelectorAll(".collapse--lbl-toggle").forEach(label => {
        label.addEventListener("keydown", e => {
            if (e.which === 32 || e.which == 13) {
                e.preventDefault();
                label.click();
            }
        });
    });

    // Make forms submit on ctrl+enter in textareas
    document.querySelectorAll("textarea").forEach(textarea => {
        textarea.addEventListener("keydown", e => {
            if (e.keyCode === 13 && e.ctrlKey) {
                textarea.form.submit();
            }
        });
    });
})();
