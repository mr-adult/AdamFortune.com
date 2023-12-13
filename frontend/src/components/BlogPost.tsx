import { useEffect, useState } from "preact/hooks";

type BlogPostProps = {
    name: string,
}

export function BlogPost(props: BlogPostProps) {
    let [html, setHtml] = useState("");
	useEffect(() => {
		fetch("/home")
			.then(data => {
                data.text().then(html => {
                    setHtml(html)
                });
			});
	}, [props.name])
	return (
		<div style='margin-left:8px;'>
            <div dangerouslySetInnerHTML={{__html: html}} />
		</div>
	);
}