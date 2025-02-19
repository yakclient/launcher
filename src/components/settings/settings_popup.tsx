import React, {useEffect, useState} from "react";
import styles from "./settings_popup.module.sass"
import {Button, Form} from "react-bootstrap";
import {loadSettings, saveSettings, UserSettings} from "@/components/settings/settings";



const Settings: React.FC = () => {
    let [settings, setSettings] = useState<UserSettings>({
        debugger: {
            enabled: false,
            suspend: false,
            port: ""
        }
    })

    useEffect(() => {
        loadSettings().then((it) => {
            console.log(it)
            setSettings(it)
        })
    }, [])

    useEffect(() => {
        saveSettings(settings as UserSettings).then(() => {

        })
    }, [settings])


    return <div id={styles.container}>
        <h1>Settings</h1>

        <div className={styles.section}>
            <h2>General</h2>

            <div>Nothing here right now..</div>
        </div>

        <div className={styles.section}>
            <h2>Advanced</h2>
            <div className={styles.section}>
                <Button
                    onClick={() => {
                        setSettings({
                            debugger: {
                                ...settings.debugger,
                                enabled: !settings.debugger.enabled,
                            },
                        })
                    }}
                    variant={settings.debugger.enabled ? "danger" : "success"}
                >
                    {settings.debugger.enabled ? "Disable Debugging Agent" : "Enable Debugging Agent"}
                </Button>
                {
                    settings.debugger.enabled ? <>
                        <Button
                            onClick={() => {
                                setSettings({
                                    debugger: {
                                        ...settings.debugger,
                                        suspend: !settings.debugger.suspend,
                                    },
                                })
                            }}
                            variant={settings.debugger.suspend ? "danger" : "success"}
                        >
                            {settings.debugger.suspend ? "Disable Suspending" : "Enable Suspending"}
                        </Button>
                        <form>
                            <Form.Label column={false}>Port:</Form.Label>
                            <Form.Control
                                onChange={(it) => {
                                    setSettings({
                                        debugger: {
                                            ...settings.debugger,
                                            port: it.target.value.length == 0 ? "5050" : it.target.value
                                        },
                                    })
                                }}
                                value={settings.debugger.port == "5050" ? "" : settings.debugger.port}
                                placeholder={"5050"}
                            />
                        </form>
                    </> : <></>
                }

            </div>
        </div>
    </div>
}

export default Settings