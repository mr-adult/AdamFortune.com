import { useEffect, useState } from "preact/hooks";
import { BlogPostDTO } from "../DTOs"
import { NavBar } from "./NavBar";

type BlogPostProps = {
    url_safe_name: string,
}

export function BlogPost(props: BlogPostProps) {
    let [html, setHtml] = useState("");
	useEffect(() => {
		fetch(`/blog_json/${props.url_safe_name}`)
			.then(response => response.json())
			.then((post: BlogPostDTO) => {
                    setHtml(post.content)
			});
	}, [props.url_safe_name])
	return (
		<>
			<NavBar additional={[]} />
			<div style='margin-left:8px;'>
         		<div dangerouslySetInnerHTML={{__html: html}} />
			</div>
		</>
	);
}