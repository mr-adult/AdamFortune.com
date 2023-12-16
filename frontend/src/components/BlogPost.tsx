import { useEffect, useState } from "preact/hooks";
import { BlogPostDTO } from "../DTOs"
import { NavBar } from "./NavBar";
import { Component, render } from "preact";
import { postProcessMD } from "../postProcessMD";

type BlogPostProps = {
    blogpost: string,
}

export class BlogPost extends Component<BlogPostProps> {
	componentDidUpdate(): void {
		postProcessMD();
	}

	render() {
	    let [html, setHtml] = useState("");
		useEffect(() => {
			fetch(`/blog_json/${this.props.blogpost}`)
				.then(response => response.json())
				.then((post: BlogPostDTO) => {
	                    setHtml(post.content)
				});
		}, [])
		return (
			<>
				<NavBar additional={[]} />
				<div style='margin-left:8px;'>
	         		<div dangerouslySetInnerHTML={{__html: html}} />
				</div>
			</>
		);
	}
}
