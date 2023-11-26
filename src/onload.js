function onLoad() {
    hljs.highlightAll();

    mermaidNodes = document.querySelectorAll("pre > code.language-mermaid");
    // the formatter I'm using doesn't quite do what mermaid is expecting, so let's fix that by moving the class "mermaid" to the "pre" element.
    for (let i = 0; i < mermaidNodes.length; i++) {                            
        mermaidNodes[i].parentNode.classList.add("mermaid");
        mermaidNodes[i].parentNode.innerHTML = mermaidNodes[i].innerHTML;
    }

}
 