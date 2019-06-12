const {parallel, src, dest} = require("gulp");
const cleanCss = require("gulp-clean-css");
const sass = require("gulp-sass");
const terser = require("gulp-terser");
const svgmin = require('gulp-svgmin');
const rename = require("gulp-rename");

async function css(){
    return await src("style/style.scss", { sourcemaps: true })
    .pipe(sass())
    .pipe(cleanCss())
    .pipe(dest("static", { sourcemaps: "." }));
};

async function js(){
    return await src("static/js/accessibility.js", { sourcemaps: true })
    .pipe(terser())
    .pipe(rename({ extname: '.min.js' }))
    .pipe(dest("static/js", { sourcemaps: "." }));
};

async function svg(){
    return await src('static/img/icons.svg')
    .pipe(svgmin({
        plugins: [{
            cleanupNumericValues: {
                floatPrecision: 1
            }
        }]
    }))
    .pipe(rename({ extname: '.min.svg' }))
    .pipe(dest('static/img'));
};

exports.default = parallel(css, js, svg);
