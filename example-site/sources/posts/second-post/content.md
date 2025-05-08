<link rel="stylesheet" href="res/post-specific.css"/>
<link rel="stylesheet" href="https://cdn.jsdelivr.net/npm/katex@0.16.8/dist/katex.min.css">
<script src="https://cdn.jsdelivr.net/npm/katex@0.16.8/dist/katex.min.js"></script>

<script>
document.addEventListener("DOMContentLoaded", () => {
       document.querySelectorAll('.math-display').forEach((el) => {
         katex.render(el.textContent, el, { throwOnError: false });
       });
     });
</script>
Look at that, post-specific CSS!

A custom tag: <GoupiCustomTag/>

Statistics class made me cry, I think this is the right formula for a gaussian? 
Either way, embedded scripts! Markdown translates one-to-one to HTML 
when you add tags. Here's KaTeX running!

(Note that markdown ``$$math$$`` blocks are translated to code blocks with
the class ``.math-display``)

$$
\frac{1}{\sqrt{2\pi} \sigma} e^{\frac{(x - \mu)^2}{2 \sigma ^2}}
$$
