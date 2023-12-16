import { useEffect, useState } from "preact/hooks";
import { RepoDTO } from "../../DTOs"
import { NavBar, NavBarButton } from "../../components/NavBar";
import { Component, ComponentChild, RenderableProps } from "preact";
import { postProcessMD } from "../../postProcessMD";

type RepoProps = {
    project: string
}

export class Repo extends Component<RepoProps> {
    componentDidUpdate(): void {
        postProcessMD();
    }

    render(): ComponentChild {
        let [html, setHtml] = useState("");
        let [additionalNav, setAdditionalNav] = useState<NavBarButton[]>([])
        useEffect(() => {
            fetch(`/projects_json/${this.props.project}`)
                .then(response => response.json())
                .then((repo: RepoDTO) => {
                    setHtml(repo.readme);
                    setAdditionalNav(repo.additional_nav_elements)
                });
        }, []);

        return (
            <>
                <NavBar additional={additionalNav} />
                <div style='margin-left:8px;'>
                    <div dangerouslySetInnerHTML={{ __html: html }} />
                </div>
            </>
        );
    }
}