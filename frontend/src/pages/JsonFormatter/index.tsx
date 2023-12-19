import { useEffect, useState } from "preact/hooks";
import { Repo } from "../Repo";

type FormattedJsonValue = {
    Ok: JsonValue
} | { 
    Err: [string, string[]]
};

type JsonValue = JsonNull | JsonBool | JsonNumber | JsonString | JsonArray | JsonObject;

type JsonNull = {
    type: "Null"
}

type JsonBool = {
    type: "Boolean",
    value: boolean,
}

type JsonNumber = {
    type: "Number",
    value: string,
}

type JsonString = {
    type: "String",
    value: string,
}

type JsonArray = {
    type: "Array",
    value: JsonValue[]
}

type JsonObject = {
    type: "Object",
    value: Record<string, JsonValue>,
}

export function JsonFormatter() {
    let [current, setCurrent] = useState<FormattedJsonValue[]>([{ Ok: { type: "Null" } }]);

    let onSubmit = function(e: any /* JSXInternal.TargetedSubmitEvent<HTMLFormElement> */) {
        console.log(e.target.elements);
        fetch("/parsejson", {
            method: "POST",
            headers: {
                "content-type": "application/json"
            },
            body: JSON.stringify({
                format: e.target.elements.format.value,
                json: e.target.elements.json.value,
            })
        })
            .then(response => response.json())
            .then(obj => {
                console.log(obj);
                setCurrent(JSON.parse(obj));
            })
            .catch(err => console.log(err))
        e.preventDefault();
    }


    return (
        <>
            <Repo project="jsonformatter" />
            <form onSubmit={onSubmit} method="post">
                <label for="type">"JSON Type:"</label><br/>
                <input type="radio" id="jsonStandard" name="format" value="JsonStandard" checked />
                <label for="jsonStandard">Standard JSON</label><br/>
                <input type="radio" id="jsonLines" name="format" value="JsonLines" />
                <label for="jsonLines">Json Lines Format</label><br/>  
                <label for="json">"JSON:"</label><br/>
                <textarea id="json" name="json" style="width:100%;min-height:200px;"></textarea><br/>
                <input type="submit" value="Submit" />
            </form>
            <br/>
            {current.map(val => 
                <div>
                {val["Ok"] !== undefined
                    ? <br/>
                    : <div>
                        {"Errors: "}
                        <ul>
                        {val["Err"][1].map(err => <li>{err}</li>)}
                        </ul>
                        <br/>
                    </div>}
                {val["Ok"] !== undefined 
                    ? <JsonValueExpander value={val["Ok"]} />
                    : <textarea style={{"width": "100%", "height": "50%"}}>{val["Err"][0]}</textarea>}
                </div>)}
        </>
    );
}

type JsonValueProps = {
    value: JsonValue
}

var idCounter = 0;

export function JsonValueExpander(props: JsonValueProps) {
    let [expanded, setExpanded] = useState(true);

    switch(true) {
        case props.value.type == "Null":
            return <span>null</span>;

        case props.value.type == "Boolean":
            return <span>{props.value.value ? "true" : "false"}</span>;

        case props.value.type == "Number":
            let id = `button${idCounter++}`;
            return <span>{props.value.value}</span>;

        case props.value.type == "String":
            return <JsonStringValue value={props.value} />;

        case props.value.type == "Array":
            return <span>
                <button onClick={() => setExpanded(!expanded)}>{expanded ? "-" : "+"}</button>
                {expanded ? "[" : "[]"}
                <div className="jsonValue" style={{borderLeft: "2px solid white"}}>
                    {props.value.value.map((val, i) => <div style={{display:expanded ? "" : "none"}}><JsonValueExpander value={val} />{(props.value as JsonArray).value.length - 1 !== i ? <br/> : ""}</div>)}
                </div>
                {expanded ? <span>{"]"}</span> : ""}
            </span>;

        case props.value.type == "Object":
            let keys = Object.keys(props.value.value);
            return <span>
                <span>
                    <button onClick={() => setExpanded(!expanded)}>{expanded ? "-" : "+"}</button>
                    {expanded ? "{" : "{}"}
                </span>
                {expanded ? <br/> : ""}
                <div className="jsonValue" style={{borderLeft: "2px solid white"}}>
                    {keys.map((key: keyof typeof props.value.value, i) =>
                        <div style={{display:expanded ? "" : "none"}}>
                            {key}: 
                            <JsonValueExpander value={(props.value as JsonObject).value[key]} />
                            {i !== keys.length - 1 ? <br/> : ""}
                        </div>
                        )
                    }
                </div>
                {expanded ? <span>{"}"}</span> : ""}
            </span>;
    }
}

function JsonStringValue(props: JsonValueProps) {
    let [value, setValue] = useState<string>((props.value as JsonString).value);
    let [formatted, setFormatted] = useState(false);

    let formatStringAsJson = function(this: string) {
        setFormatted(true);
        let button = document.getElementById(this);
        let sibling = (button.parentNode as Element).childNodes[0] as Element;
        let innerHtml = sibling.innerHTML;
        innerHtml = innerHtml.substring(1, innerHtml.length - 1)
            .split("\\\"")
            .join("\"");
        // strip the wrapping quotes
        console.log("sending: " + innerHtml);

        fetch("/formatjson", {
            method: "POST",
            headers: {
                "content-type": "application/json"
            },
            body: JSON.stringify(innerHtml)
        })
            .then(response => response.json())
            .then(returnVal => {
                let returnValCleansed = returnVal.split("\r\n").join("<br/>")
                    .split("\n").join("<br/>")
                    .split("\t").join("&nbsp;&nbsp;&nbsp;&nbsp;");

                setValue(returnValCleansed);
            })
            .catch(err => console.log(err))
    } 
    
    let id = `button${idCounter++}`;
    return <span>
        <span dangerouslySetInnerHTML={{__html: value}}></span>
        {!formatted 
            ? <button id={id} onClick={formatStringAsJson.bind(id)}>Format as JSON</button> 
            : ""}
    </span>;

}