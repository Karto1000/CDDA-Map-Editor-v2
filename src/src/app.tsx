import React, {createContext, useEffect, useRef, useState} from 'react';

import "./app.scss"
import {TauriCommand, TauriEvent, ToastType} from "./tauri/events/types.js";
import {TabTypeKind, useTabs, UseTabsReturn} from './shared/hooks/useTabs.ts';
import {NoTabScreen} from "./shared/components/noTabScreen.js";
import {Header} from "./shared/components/header.js";
import {getColorFromTheme, Theme, useTheme} from "./shared/hooks/useTheme.js";
import {ProgramData} from "./tauri/types/editor.js";
import {useEditorData} from "./shared/hooks/useEditorData.js";
import {MainCanvas} from "./shared/components/mainCanvas.js";
import {useWindows} from "./shared/hooks/useWindows.js";
import {tauriBridge} from "./tauri/events/tauriBridge.js";
import {useThreeSetup} from "./features/three/hooks/useThreeSetup.js";
import {useTileset} from "./features/sprites/hooks/useTileset.js";
import toast, {ToastBar, Toaster} from "react-hot-toast";
import Icon, {IconName} from "./shared/components/icon.js";
import {useTauriEvent} from "./shared/hooks/useTauriEvent.js";
import {MapViewer} from "./features/viewer/mapViewer.js";
import {openWindow, WindowLabel} from "./windows/lib.js";
import {useKeybindings} from "./shared/hooks/useKeybindings.js";
import {MapEditor} from "./features/editor/mapEditor.js";

export const ThemeContext = createContext<{ theme: Theme }>({
    theme: Theme.Dark,
});

export type SidebarContent = {
    chosenProperties: React.JSX.Element,
    calculatedParameters: React.JSX.Element,
}

export const TabContext = createContext<UseTabsReturn | null>(null)
export const EditorDataContext = createContext<ProgramData | null>(null)

function App() {
    const eventBus = useRef<EventTarget>(new EventTarget())
    const [theme] = useTheme(eventBus);

    const canvasContainerRef = useRef<HTMLDivElement>(null)
    const canvasRef = useRef<HTMLCanvasElement>(null);
    const showGridRef = useRef<boolean>(true)

    const {threeConfigRef} = useThreeSetup(theme, canvasRef, canvasContainerRef)
    const editorData = useEditorData()
    const tabs = useTabs(eventBus)
    const {spritesheetConfig, tilesheets} = useTileset(eventBus, threeConfigRef)
    const {importMapWindowRef, settingsWindowRef, newMapWindowRef, aboutWindowRef} = useWindows()

    const [isAppReady, setIsAppReady] = useState<boolean>(false)

    useKeybindings(
        window,
        eventBus,
        editorData,
        [tabs]
    )

    useEffect(() => {
        (async () => {
            await tauriBridge.invoke(TauriCommand.FRONTEND_READY, {})
        })()
    }, []);

    useEffect(() => {
        if (!editorData) return;

        if (!editorData.config.cdda_path) {
            openWindow(WindowLabel.Welcome, theme, {defaultWidth: 760, defaultHeight: 600})
            return
        }

        setIsAppReady(true)
    }, [editorData]);

    useTauriEvent(
        TauriEvent.EMIT_TOAST_MESSAGE,
        (d) => {
            if (d.type === ToastType.Error) {
                toast.error(d.message)
            }

            if (d.type === ToastType.Success) {
                toast.success(d.message)
            }
        },
        []
    )

    function getMainBasedOnTab(): React.JSX.Element {
        if (tabs.openedTab !== null) {
            if (tabs.tabs[tabs.openedTab].tab_type === TabTypeKind.LiveViewer)
                return <MapViewer
                    showGridRef={showGridRef}
                    eventBus={eventBus}
                    tilesheets={tilesheets}
                    spritesheetConfig={spritesheetConfig}
                    threeConfig={threeConfigRef}
                    canvas={{
                        canvasRef,
                        canvasContainerRef
                    }}
                />

            if (tabs.tabs[tabs.openedTab].tab_type === TabTypeKind.MapEditor)
                return <MapEditor
                    showGridRef={showGridRef}
                    eventBus={eventBus}
                    tilesheets={tilesheets}
                    spritesheetConfig={spritesheetConfig}
                    threeConfig={threeConfigRef}
                    canvas={{
                        canvasRef,
                        canvasContainerRef
                    }}
                />
        }

        return <NoTabScreen importMapWindowRef={importMapWindowRef} newMapWindowRef={newMapWindowRef}/>
    }

    return (
        <div className={`app ${theme}-theme`}>
            {!isAppReady && <div className={"loading-screen"}>
                <div className={"loader"}/>
            </div>}
            <Toaster
                position={"bottom-right"}
                toastOptions={{
                    style: {
                        borderRadius: 0,
                        maxWidth: "100%",
                    },
                    success: {
                        icon: <Icon name={IconName.CheckmarkMedium}/>,
                        style: {
                            background: getColorFromTheme(theme, "darkBlue"),
                            border: `2px solid ${getColorFromTheme(theme, "selected")}`,
                        }
                    },
                    error: {
                        icon: <Icon name={IconName.CloseSmall}/>,
                        duration: 5000,
                        style: {
                            background: getColorFromTheme(theme, "delete"),
                            border: `2px solid ${getColorFromTheme(theme, "lightDelete")}`,
                        }
                    }
                }}>
                {(t: any) => <ToastBar toast={t}/>}
            </Toaster>
            <EditorDataContext.Provider value={editorData}>
                <ThemeContext.Provider value={{theme}}>
                    <TabContext.Provider value={tabs}>
                        <Header
                            showGrid={showGridRef}
                            importMapWindowRef={importMapWindowRef}
                            settingsWindowRef={settingsWindowRef}
                            eventBus={eventBus}
                            newMapWindowRef={newMapWindowRef}
                            aboutWindowRef={aboutWindowRef}
                        />

                        <MainCanvas
                            canvasRef={canvasRef}
                            canvasContainerRef={canvasContainerRef}
                            displayState={tabs.shouldDisplayCanvas() ? "flex" : "none"}
                        />
                        {getMainBasedOnTab()}
                    </TabContext.Provider>
                </ThemeContext.Provider>
            </EditorDataContext.Provider>
        </div>
    );
}

export default App;
