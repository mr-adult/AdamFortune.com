import { useEffect, useState } from 'preact/hooks';
import './style.css';

export function Projects() {
    let [repos, setRepos] = useState([]);

    useEffect(() => {
        fetch("/projects", {
            method: "GET",
        })
        .then((response) => response.json())
        .then((data: Repo[]) => {
            setRepos(data);
        })
        .catch((error) => console.log(error));
    }, []);

    let projects = repos;
    let i = 0;
	return (
        <ul style={'display: grid; column-count: 2; column-gap: 20px; row-gap: 20px; padding: 0px; word-break: break-word'}>
            {projects.map(repo => {
                <RepoCard repo={repo} index={++i} />
            })}
        </ul>
	);
}

type RepoCardProps = {
    repo: Repo,
    index: number,
}

function RepoCard(props: RepoCardProps) {
    let style = `grid-row: ${props.index}; grid-column: 1;`
    let href = `/projects/${props.repo.name}`;
    return (
        <li style={style}>
            <h2>
                <a href={href}>{props.repo.name}</a>
            </h2>
            <p>{props.repo.description}</p>
        </li>
    );
}
