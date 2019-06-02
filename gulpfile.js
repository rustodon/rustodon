const {series, src, dest} = require("gulp");
const cleanCss = require("gulp-clean-css");
const sass = require("gulp-sass");

function defaultTask(){
    return src("style/style.scss", { sourcemaps: true })
    .pipe(sass())
    .pipe(cleanCss())
    .pipe(dest("static", { sourcemaps: "." }))
}

exports.default = series(defaultTask);