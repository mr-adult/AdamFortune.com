import { useEffect, useState } from 'preact/hooks';
import { RepoDTO } from '../../DTOs';
import { NavBar } from '../../components/NavBar';

export function Projects() {
    let [repos, setRepos] = useState<RepoDTO[]>([]);

    useEffect(() => {
        fetch("/projects_json", {
            method: "GET",
        })
            .then((response) => response.json())
            .then((data: RepoDTO[]) => {
                setRepos(data);
            })
            .catch((error) => console.log(error));
    }, []);

    let i = 0;
	return (
        <>
            <NavBar additional={[]} />
            <ul className="contentList">
                {repos.map(repo => {
                    return <RepoCard repo={repo} index={++i} />
                })}
            </ul>
        </>
	);
}

type RepoCardProps = {
    repo: RepoDTO,
    index: number,
}

function RepoCard(props: RepoCardProps) {
    let style = `grid-row: ${props.index}; grid-column: 1;`
    let href = `/projects/${props.repo.url_safe_name}`;
    return (
        <li className="contentItem" style={style}>
            <h2>
                <a href={href}>{props.repo.name}</a>
            </h2>
            <p>{props.repo.description}</p>
        </li>
    );
}
