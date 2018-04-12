// this makes <enter> and <space> toggle hidden-checkbox-based collapsables.
(function() {
    var labels = document.querySelectorAll('.collapse--lbl-toggle');

    for (var i=0; i<labels.length; i++) {
        var label = labels[i];
        label.addEventListener('keydown', bindListener(label));

        function bindListener(label) {
            return function(e) {
                if (e.which === 32 || e.which == 13) {
                    e.preventDefault();
                    label.click();
                }
            }
        }
    }
})();
