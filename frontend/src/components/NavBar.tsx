import { useLocation } from 'preact-iso';
import { NavBarElement } from '../DTOs';

export type NavBarProps = {
    additional: NavBarButton[]
}

export interface NavBarButton extends NavBarElement {
    display_text: string,
    href: string,
}

export function NavBar(props: NavBarProps) {
    let buttons: NavBarButton[] = [
        {
            display_text: "Home",
            href: "/",
        },
        {
            display_text: "Projects",
            href: "/projects",
        },
        {
            display_text: "Blog",
            href: "/blog",
        },
    ];

    for (const additional of props.additional) {
        buttons.push(additional);
    }

	const { url } = useLocation();

	return (
        <nav id='navbar'>
            <ul id='navbar_list' style='list-style: none; display: flex; flex-direction: row; justify-content: space-around; margin: 0px; padding: 0px;'>
                {buttons.map(button => {
                    return <li>
                        <a href={button.href}>{button.display_text}</a>
                    </li>
                })}
            </ul>
        </nav>
    );
}