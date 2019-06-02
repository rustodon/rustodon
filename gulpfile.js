const {parallel, src, dest} = require("gulp");
const cleanCss = require("gulp-clean-css");
const sass = require("gulp-sass");
const terser = require("gulp-terser");
const rename = require("gulp-rename");

async function css(){
    return await src("style/style.scss", { sourcemaps: true })
    .pipe(sass())
    .pipe(cleanCss())
    .pipe(dest("static", { sourcemaps: "." }));
}

async function js(){
    return await src("static/js/accessibility.js", { sourcemaps: true })
    .pipe(terser())
    .pipe(rename({ extname: '.min.js' }))
    .pipe(dest("static/js", { sourcemaps: "." }));
}

exports.default = parallel(css, js);