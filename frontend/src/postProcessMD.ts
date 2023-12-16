import mermaid from "mermaid";

export function postProcessMD() {
	// @ts-expect-error - handled by imports in index.html
	hljs.highlightAll();
	
	const mermaidNodes = document.querySelectorAll("pre > code.language-mermaid");
	// the formatter I'm using doesn't quite do what mermaid is expecting, so let's fix that by moving the class "mermaid" to the "pre" element.
	for (let i = 0; i < mermaidNodes.length; i++) {                            
		(mermaidNodes[i].parentNode as Element).classList.add("mermaid");
		(mermaidNodes[i].parentNode as Element).innerHTML = mermaidNodes[i].innerHTML;
	}

	mermaid.init(undefined, document.querySelector("pre.mermaid") as HTMLElement);
}