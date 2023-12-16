import { useEffect, useState } from 'preact/hooks';
import { BlogPostDTO } from '../../DTOs';
import { NavBar } from '../../components/NavBar';

export function Blog() {
    let [posts, setPosts] = useState<BlogPostDTO[]>([]);

    useEffect(() => {
        fetch("/blog_json", {
            method: "GET",
        })
            .then((response) => response.json())
            .then((data: BlogPostDTO[]) => {
                setPosts(data);
            })
            .catch((error) => console.log(error));
    }, []);

    let i = 0;
	return (
        <>
			<NavBar additional={[]} />
            <ul className="contentList">
                {posts.map(post => {
                    return <BlogCard post={post} index={++i} />
                })}
            </ul>
        </>
	);
}

type BlogCardProps = {
    post: BlogPostDTO,
    index: number,
}

function BlogCard(props: BlogCardProps) {
    let style = `grid-row: ${props.index}; grid-column: 1;`
    let href = `/blog_json/${props.post.url_safe_name}`;
    return (
        <li className="contentItem" style={style}>
            <h2>
                <a href={href}>{props.post.name}</a>
            </h2>
            <p>{props.post.description}</p>
        </li>
    );
}
